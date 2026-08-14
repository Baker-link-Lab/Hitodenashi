use std::io::Write;
use std::path::PathBuf;

use nanomp3::{Decoder, MAX_SAMPLES_PER_FRAME};

const TARGET_RATE: u32 = 11025;

fn main() {
    println!("cargo:rerun-if-changed=data/sounds-of-a-dangerous-siren.mp3");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let raw_path = out_dir.join("siren_11k_s16le.raw");

    let mp3 = std::fs::read("data/sounds-of-a-dangerous-siren.mp3")
        .expect("data/sounds-of-a-dangerous-siren.mp3 が読み込めません");

    let mut decoder = Decoder::new();
    let mut offset = 0usize;
    let mut pcm_buf = [0.0f32; MAX_SAMPLES_PER_FRAME];
    let mut mono_samples: Vec<f32> = Vec::new();
    let mut source_rate = 44100u32;

    while offset < mp3.len() {
        let (consumed, info) = decoder.decode(&mp3[offset..], &mut pcm_buf);
        if consumed == 0 {
            break;
        }
        offset += consumed;

        if let Some(frame) = info {
            source_rate = frame.sample_rate;
            let channels = frame.channels.num() as usize;
            for chunk in pcm_buf[..frame.samples_produced].chunks_exact(channels) {
                mono_samples.push(chunk.iter().sum::<f32>() / channels as f32);
            }
        }
    }

    assert!(!mono_samples.is_empty(), "MP3 のデコードに失敗しました");

    // ナイキスト以下でカットするアンチエイリアス FIR フィルタを適用
    let cutoff = TARGET_RATE as f64 * 0.45;
    let mut filtered = lowpass_fir(&mono_samples, cutoff, source_rate as f64, 63);

    // ピーク正規化：最大振幅を full-scale に拡大して音量を最大化
    let peak = filtered.iter().fold(0.0f32, |a, &s| a.max(s.abs()));
    if peak > 1e-6 {
        let gain = 0.95 / peak; // 5% のクリッピングマージンを残す
        for s in &mut filtered {
            *s *= gain;
        }
    }

    // 線形補間でリサンプリング
    let ratio = source_rate as f64 / TARGET_RATE as f64;
    let output_len = (filtered.len() as f64 / ratio).floor() as usize;

    let mut out_file = std::fs::File::create(&raw_path).expect("出力ファイルの作成に失敗しました");

    for i in 0..output_len {
        let pos = i as f64 * ratio;
        let idx = pos as usize;
        let frac = pos - idx as f64;
        let a = filtered.get(idx).copied().unwrap_or(0.0) as f64;
        let b = filtered.get(idx + 1).copied().unwrap_or(0.0) as f64;
        let sample_i16 = ((a + (b - a) * frac).clamp(-1.0, 1.0) * 32767.0) as i16;
        out_file.write_all(&sample_i16.to_le_bytes()).unwrap();
    }

    eprintln!(
        "[build] {:.1}s @ {}Hz -> {:.1}s @ {}Hz ({} bytes)",
        mono_samples.len() as f32 / source_rate as f32,
        source_rate,
        output_len as f32 / TARGET_RATE as f32,
        TARGET_RATE,
        output_len * 2
    );
}

/// 63-tap Hann 窓シンク FIR ローパスフィルタ
fn lowpass_fir(samples: &[f32], cutoff_hz: f64, sample_rate_hz: f64, num_taps: usize) -> Vec<f32> {
    use std::f64::consts::PI;
    let cutoff = cutoff_hz / sample_rate_hz;
    let m = (num_taps - 1) as f64 / 2.0;

    let mut taps: Vec<f64> = (0..num_taps)
        .map(|i| {
            let n = i as f64 - m;
            let hann = 0.5 * (1.0 - (2.0 * PI * i as f64 / (num_taps - 1) as f64).cos());
            let sinc = if n.abs() < 1e-10 {
                2.0 * cutoff
            } else {
                (2.0 * PI * cutoff * n).sin() / (PI * n)
            };
            hann * sinc
        })
        .collect();

    let gain: f64 = taps.iter().sum();
    for t in &mut taps {
        *t /= gain;
    }

    let half = num_taps / 2;
    let n = samples.len();
    (0..n)
        .map(|i| {
            taps.iter().enumerate().fold(0.0f64, |acc, (j, &tap)| {
                let k = i as isize + j as isize - half as isize;
                let s = if k >= 0 && (k as usize) < n {
                    samples[k as usize] as f64
                } else {
                    0.0
                };
                acc + tap * s
            }) as f32
        })
        .collect()
}

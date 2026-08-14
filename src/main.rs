#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp2040_hal as hal;

use hal::pac;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use embedded_hal::pwm::SetDutyCycle;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// build.rs が生成する 11kHz mono s16le PCM
const SIREN_RAW: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/siren_11k_s16le.raw"));

fn play_siren<C: SetDutyCycle>(timer: &mut rp2040_hal::Timer, ch: &mut C) {
    let max = ch.max_duty_cycle() as u32;
    let mut next: u32 = timer.get_counter_low();

    for chunk in SIREN_RAW.chunks_exact(2) {
        next = next.wrapping_add(91); // 11025Hz ≈ 90.7µs
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        let duty = ((sample as i32 + 32768) as u32 * max / 65535) as u16;
        // 目標時刻まで待機してから書き込む（ジッター最小化）
        while (timer.get_counter_low().wrapping_sub(next) as i32).is_negative() {}
        ch.set_duty_cycle(duty).ok();
    }
    ch.set_duty_cycle((max / 2) as u16).ok();
}

#[rp2040_hal::entry]
fn main() -> ! {
    info!("Program start!");
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut clip = pins.gpio0.into_pull_down_input();

    // GPIO28 = PWM スライス6 チャンネルA
    // set_top(2047): 11-bit 解像度、125MHz / 2048 ≈ 61kHz キャリア（可聴域外）
    let pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let mut pwm6 = pwm_slices.pwm6;
    pwm6.set_top(2047u16);
    pwm6.enable();
    let mut channel_a = pwm6.channel_a;
    channel_a.output_to(pins.gpio28);
    channel_a.set_duty_cycle(1024).ok(); // 初期値 = 無音（中点）

    loop {
        let state = clip.is_high().unwrap();
        info!("state: {}", state);

        if !state {
            play_siren(&mut timer, &mut channel_a);
        }

        timer.delay_ms(50);
    }
}

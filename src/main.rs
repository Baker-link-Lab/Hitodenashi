#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp2040_hal as hal;

use hal::pac;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

const BASS_TAB: [u32; 16] = [
    700, 660, 620, 580, 540, 500, 460, 420, 420, 460, 500, 540, 580, 620, 660, 700,
];

fn play_note<T: OutputPin>(timer: &mut rp2040_hal::Timer, speaker: &mut T, note_index: usize) {
    for _ in 0..45 {
        speaker.set_high().unwrap();
        timer.delay_us(BASS_TAB[note_index]);
        speaker.set_low().unwrap();
        timer.delay_us(BASS_TAB[note_index]);
    }
}

fn play_melody<T: OutputPin>(timer: &mut rp2040_hal::Timer, speaker: &mut T) {
    for note_index in 0..BASS_TAB.len() {
        play_note(timer, speaker, note_index);
    }
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
    let mut speaker = pins.gpio28.into_push_pull_output();
    speaker.set_low().unwrap();

    loop {
        let state = clip.is_high().unwrap();
        info!("state: {}", state);

        if !state {
            play_melody(&mut timer, &mut speaker);
        }

        timer.delay_ms(50);
    }
}

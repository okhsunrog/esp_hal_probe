#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{peripherals::TIMG0, timer::timg::TimerGroup, Blocking};
#[allow(unused)]
use {defmt_rtt as _, esp_backtrace as _};

#[embassy_executor::task]
async fn run() {
    loop {
        info!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    info!("Before peripheral init");
    let peripherals = esp_hal::init(esp_hal::Config::default());
    info!("Init!");
    // RustRover shows error if I don't write full type here, that's weird
    let timg0: TimerGroup<TIMG0, Blocking> = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    spawner.spawn(run()).ok();

    loop {
        info!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

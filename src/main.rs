#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    system::SystemControl,
    timer::timg::TimerGroup,
};

#[embassy_executor::task]
async fn run() {
    loop {
        info!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    info!("Init!");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0.timer0);

    spawner.spawn(run()).ok();

    loop {
        // info!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

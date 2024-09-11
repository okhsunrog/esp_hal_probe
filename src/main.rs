#![no_std]
#![no_main]

extern crate alloc;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    peripherals::TIMG0,
    rng::Rng,
    timer::{
        systimer::{SystemTimer, Target},
        timg::TimerGroup,
    },
    Blocking,
};
use esp_wifi::{ble::controller::asynch::BleConnector, initialize, EspWifiInitFor};
use static_cell::StaticCell;
#[allow(unused)]
use {defmt_rtt as _, esp_alloc as _, esp_backtrace as _};

#[embassy_executor::task]
async fn run() {
    loop {
        info!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(128 * 1024);
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // RustRover shows error if I don't write full type here, that's weird
    let timg0: TimerGroup<TIMG0, Blocking> = TimerGroup::new(peripherals.TIMG0);

    let init = unwrap!(initialize(
        EspWifiInitFor::Ble,
        timg0.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    ));
    let systimer = SystemTimer::new(peripherals.SYSTIMER).split::<Target>();
    esp_hal_embassy::init(systimer.alarm0);

    spawner.spawn(run()).ok();

    loop {
        info!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

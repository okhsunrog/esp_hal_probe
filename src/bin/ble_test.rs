#![no_std]
#![no_main]

use bt_hci::controller::ExternalController;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    rng::Rng,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_wifi::ble::controller::BleConnector;
use trouble_example_apps::ble_bas_peripheral;
#[allow(unused)]
use {defmt_rtt as _, esp_alloc as _, esp_backtrace as _};

#[embassy_executor::task]
async fn run() {
    loop {
        info!("Hello world from embassy using esp-hal-async!");
        // replace the log with blinking led?
        Timer::after(Duration::from_millis(2_000)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_alloc::heap_allocator!(128 * 1024);
    let peripherals = esp_hal::init(esp_hal::Config::default());
    info!("Initializing...");

    // RustRover shows error if I don't write full type here, that's weird
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let init = esp_wifi::init(
        timg0.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);
    spawner.spawn(run()).unwrap();

    let mut bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, &mut bluetooth);
    let controller: ExternalController<_, 20> = ExternalController::new(connector);
    info!("Starting BLE...");
    ble_bas_peripheral::run(controller).await;
}

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::{GpioPin, Input, Io, Pull},
    peripherals::Peripherals,
    prelude::*,
    rtc_cntl::Rtc,
    system::SystemControl,
    timer::{timg::TimerGroup, OneShotTimer},
};
use static_cell::make_static;

#[embassy_executor::task]
async fn monitor_pins(
    mut input4: Input<'static, GpioPin<4>>,
    mut input5: Input<'static, GpioPin<5>>,
) {
    loop {
        match select(input4.wait_for_rising_edge(), input5.wait_for_rising_edge()).await {
            Either::First(_) => info!("Rising edge detected on Pin 4!"),
            Either::Second(_) => info!("Rising edge detected on Pin 5!"),
        }
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Enable the RWDT watchdog timer:
    let mut rtc = Rtc::new(peripherals.LPWR, None);
    rtc.rwdt.set_timeout(2.secs());
    rtc.rwdt.enable();
    info!("RWDT watchdog enabled!");

    // Initialize the SYSTIMER peripheral, and then Embassy:
    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let timers = [OneShotTimer::new(timg0.timer0.into())];
    let timers = make_static!(timers);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let input4: Input<GpioPin<4>> = Input::new(io.pins.gpio4, Pull::Down);
    let input5: Input<GpioPin<5>> = Input::new(io.pins.gpio5, Pull::Down);
    esp_hal_embassy::init(&clocks, timers);
    info!("Embassy initialized!");

    spawner.spawn(monitor_pins(input4, input5)).unwrap();

    // Periodically feed the RWDT watchdog timer when our tasks are not running:
    loop {
        rtc.rwdt.feed();
        Timer::after(Duration::from_secs(1)).await;
    }
}

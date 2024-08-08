#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::{GpioPin, Input, Io, Pull},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    timer::{timg::TimerGroup, OneShotTimer},
};
use static_cell::make_static;

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Initialize the SYSTIMER peripheral, and then Embassy:
    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let timers = make_static!([OneShotTimer::new(timg0.timer0.into())]);
    esp_hal_embassy::init(&clocks, timers);
    info!("Embassy initialized!");

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut input4: Input<GpioPin<4>> = Input::new(io.pins.gpio4, Pull::None);
    let mut input5: Input<GpioPin<5>> = Input::new(io.pins.gpio5, Pull::None);

    loop {
        match select(input4.wait_for_rising_edge(), input5.wait_for_rising_edge()).await {
            Either::First(_) => info!("Rising edge detected on Pin 4!"),
            Either::Second(_) => info!("Rising edge detected on Pin 5!"),
        }
    }
}

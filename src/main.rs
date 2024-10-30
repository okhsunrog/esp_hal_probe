#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
#[allow(unused)]
use esp_backtrace as _;
use esp_hal::{peripherals::TIMG0, timer::timg::TimerGroup, Blocking};
use rtt_target::{rtt_init, ChannelMode::*};

#[embassy_executor::task]
async fn run() {
    loop {
        info!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let channels = rtt_init! {
        up: {
            0: {
                size: 512,
                mode: BlockIfFull,
            }
            1: {
                size: 512,
                mode: BlockIfFull,
            }
            2: {
                size: 2048,
                mode: BlockIfFull,
            }
        }
        down: {
            0: {
                size: 512,
                mode: BlockIfFull,
            }
        }
    };
    rtt_target::set_defmt_channel(channels.up.0);
    rtt_target::set_print_channel(channels.up.1);
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

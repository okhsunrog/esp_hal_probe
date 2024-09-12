#![no_std]
#![no_main]

extern crate alloc; // do I need this?

use bt_hci::controller::ExternalController;
use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
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
use trouble_host::{
    advertise::{AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE},
    attribute::{AttributeTable, CharacteristicProp, Service, Uuid},
    Address,
    BleHost,
    BleHostResources,
    PacketQos,
};
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
    info!("Initializing...");

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

    let mut bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, &mut bluetooth);
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    static HOST_RESOURCES: StaticCell<BleHostResources<8, 8, 256>> = StaticCell::new();
    let host_resources = HOST_RESOURCES.init(BleHostResources::new(PacketQos::None));

    let mut ble: BleHost<'_, _> = BleHost::new(controller, host_resources);

    ble.set_random_address(Address::random([0xFF, 0x9F, 0x1A, 0x05, 0xE4, 0xFF]));
    let mut table: AttributeTable<'_, NoopRawMutex, 10> = AttributeTable::new();

    let id = b"Trouble ESP32";
    let appearance = [0x80, 0x07];
    let mut bat_level = [0; 1];
    // Generic Access Service (mandatory)
    let mut svc = table.add_service(Service::new(0x1800));
    let _ = svc.add_characteristic_ro(0x2A00, id);
    let _ = svc.add_characteristic_ro(0x2A01, &appearance[..]);
    svc.build();

    // Generic attribute service (mandatory)
    table.add_service(Service::new(0x1801));

    // Battery service
    let bat_level_handle = table
        .add_service(Service::new(0x180F))
        .add_characteristic(
            0x2A19,
            &[CharacteristicProp::Read, CharacteristicProp::Notify],
            &mut bat_level,
        )
        .build();

    let mut adv_data = [0; 31];
    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[Uuid::Uuid16([0x0F, 0x18])]),
            AdStructure::CompleteLocalName(b"Trouble ESP32"),
        ],
        &mut adv_data[..],
    )
    .unwrap();

    let server = ble.gatt_server::<NoopRawMutex, 10, 256>(&table);

    spawner.spawn(run()).ok();
    info!("Running!");

    info!("Starting advertising and GATT service");
    // Run all 3 tasks in a join. They can also be separate embassy tasks.
    let _ = join3(
        // Runs the BLE host task
        ble.run(),
        // Processing events from GATT server (if an attribute was written)
        async {
            loop {
                match server.next().await {
                    Ok(_event) => {
                        info!("Gatt event!");
                    }
                    Err(e) => {
                        warn!("Error processing GATT events: {:?}", e);
                    }
                }
            }
        },
        // Advertise our presence to the world.
        async {
            loop {
                let mut advertiser = ble
                    .advertise(
                        &Default::default(),
                        Advertisement::ConnectableScannableUndirected {
                            adv_data: &adv_data[..],
                            scan_data: &[],
                        },
                    )
                    .await
                    .unwrap();
                let conn = advertiser.accept().await.unwrap();
                // Keep connection alive and notify with value change
                let mut tick: u8 = 0;
                loop {
                    if !conn.is_connected() {
                        break;
                    }
                    Timer::after(Duration::from_secs(1)).await;
                    tick = tick.wrapping_add(1);
                    server
                        .notify(&ble, bat_level_handle, &conn, &[tick])
                        .await
                        .ok();
                }
            }
        },
    )
    .await;
}

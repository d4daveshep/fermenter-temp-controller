#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::{
    Async,
    interrupt::software::SoftwareInterruptControl,
    timer::timg::TimerGroup,
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx},
};

esp_bootloader_esp_idf::esp_app_desc!();

// `sensors`/`relays` (in lib.rs) are unused by the echo_task main loop below
// until task 16 wires them into the real control loop; exercised for now by
// examples/discover_sensors.rs and examples/relay_test.rs.
#[allow(unused_imports)]
use esp32c3::{relays, sensors};

/// Hardware bring-up echo loop (tasks.md task 10): confirms the toolchain,
/// flash, and async USB-Serial-JTAG I/O all work before any protocol logic
/// is layered on top. Deliberately a single task rather than split
/// reader/writer tasks — see design.md Decision 2's single-task-for-v1
/// architecture, which this file's eventual replacement (task 16) also
/// follows.
#[embassy_executor::task]
async fn echo_task(
    mut rx: UsbSerialJtagRx<'static, Async>,
    mut tx: UsbSerialJtagTx<'static, Async>,
) {
    #[cfg(feature = "debug-log")]
    esp_println::println!("echo_task: started");

    let mut buf = [0u8; 64];
    loop {
        match rx.read(&mut buf).await {
            Ok(0) => {}
            Ok(len) => {
                if tx.write_all(&buf[..len]).await.is_ok() {
                    let _ = tx.flush().await;
                } else {
                    #[cfg(feature = "debug-log")]
                    esp_println::println!("echo_task: write error");
                }
            }
            #[allow(unreachable_patterns)]
            Err(_e) => {
                #[cfg(feature = "debug-log")]
                esp_println::println!("echo_task: read error: {:?}", _e);
            }
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Claims TIMG0 and the FROM_CPU0 software interrupt as the Embassy/RTOS
    // runtime's resources — see design.md Decision 2 and its Open Questions
    // entry on what this reserves.
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    let (rx, tx) = UsbSerialJtag::new(peripherals.USB_DEVICE)
        .into_async()
        .split();

    spawner.spawn(echo_task(rx, tx).unwrap());
}

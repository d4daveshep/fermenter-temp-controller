#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};
use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::{
    Async,
    interrupt::software::SoftwareInterruptControl,
    timer::timg::TimerGroup,
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx},
};
use logic::TelemetryData;

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
///
/// Also exercises the task 14 target frame parser: `frame_buf`/
/// `frame_in_progress` are plain locals that persist for the lifetime of
/// this never-returning task, giving `logic::parse_target_frame` the
/// across-tick accumulator design.md Decision 11 calls for, so a `<19.5>`
/// frame split across two `rx.read` calls isn't lost.
///
/// And the task 15 JSON telemetry emission: `select`s between `rx.read` and
/// a 10-second `Ticker` so periodic emission doesn't require RX activity
/// (a plain sequential `Timer::after` re-armed each loop pass would instead
/// reset on every incoming byte, back to a full 10s of RX silence). Still
/// one task per design.md Decision 2 — `select` interleaves the two
/// concerns rather than splitting them into separate tasks. The emitted
/// values are fixed placeholders for now — real sensor reads and a real
/// `make_action_decision` call are task 16's job; this step only proves the
/// periodic-emission wiring and that the emitted JSON is well-formed
/// against the host's `Reading` deserialiser.
#[embassy_executor::task]
async fn echo_task(
    mut rx: UsbSerialJtagRx<'static, Async>,
    mut tx: UsbSerialJtagTx<'static, Async>,
) {
    #[cfg(feature = "debug-log")]
    esp_println::println!("echo_task: started");

    let mut buf = [0u8; 64];
    let mut frame_buf: heapless::Vec<u8, 16> = heapless::Vec::new();
    let mut frame_in_progress = false;
    let mut ticker = Ticker::every(Duration::from_secs(10));

    loop {
        match select(rx.read(&mut buf), ticker.next()).await {
            Either::First(Ok(0)) => {}
            Either::First(Ok(len)) => {
                for &byte in &buf[..len] {
                    if byte == b'<' {
                        frame_buf.clear();
                        frame_in_progress = true;
                    }
                    if frame_in_progress {
                        if frame_buf.push(byte).is_err() {
                            // Oversized frame - abandon it and wait for the
                            // next `<` (mirrors the Arduino's `ndx` clamp:
                            // bytes past capacity are dropped, not grown
                            // unbounded — see design.md Decision 11).
                            frame_buf.clear();
                            frame_in_progress = false;
                        } else if byte == b'>' {
                            let parsed = logic::parse_target_frame(&frame_buf);
                            #[cfg(feature = "debug-log")]
                            if let Some(value) = parsed {
                                esp_println::println!("target updated: {value}");
                            }
                            #[cfg(not(feature = "debug-log"))]
                            let _ = parsed;
                            frame_buf.clear();
                            frame_in_progress = false;
                        }
                    }
                }

                if tx.write_all(&buf[..len]).await.is_ok() {
                    let _ = tx.flush().await;
                } else {
                    #[cfg(feature = "debug-log")]
                    esp_println::println!("echo_task: write error");
                }
            }
            #[allow(unreachable_patterns)]
            Either::First(Err(_e)) => {
                #[cfg(feature = "debug-log")]
                esp_println::println!("echo_task: read error: {:?}", _e);
            }
            Either::Second(()) => {
                let telemetry = logic::format_telemetry(&TelemetryData {
                    target: 19.5,
                    average: 18.2,
                    min: 18.0,
                    max: 18.4,
                    ambient: 20.1,
                    action: "Rest",
                    reason_code: "RC3.1",
                });
                let _ = tx.write_all(telemetry.as_bytes()).await;
                let _ = tx.flush().await;
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

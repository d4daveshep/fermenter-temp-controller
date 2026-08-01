#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};
use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::{
    Async,
    gpio::{Flex, Level, Output, OutputConfig},
    interrupt::software::SoftwareInterruptControl,
    timer::timg::TimerGroup,
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx},
};
use esp32c3::{relays::RelayController, sensors::SensorReader};
use logic::{
    Action, ControllerActionRules, DEFAULT_TARGET_TEMP, TARGET_RANGE, TelemetryData,
    TemperatureReadings,
};

esp_bootloader_esp_idf::esp_app_desc!();

// design.md Decision 9.
const FERMENTER_EMA_WINDOW: u32 = 60;
const AMBIENT_EMA_WINDOW: u32 = 10;
const TELEMETRY_EVERY_N_TICKS: u32 = 10;

/// The full control loop (tasks.md task 16): one task per design.md
/// Decision 2, `select`ing each iteration between USB-Serial-JTAG RX (target
/// frame parsing, task 14) and a 1-second `Ticker` that drives the real
/// control cycle — read both sensors, update EMAs, decide, drive the
/// relays, and emit JSON telemetry every 10th tick (task 15).
///
/// `frame_buf`/`frame_in_progress` (Decision 11) and `current_action` are
/// plain locals that persist for this never-returning task's lifetime,
/// standing in for the Arduino's static locals and its global
/// `currentAction`.
#[embassy_executor::task]
async fn control_task(
    mut rx: UsbSerialJtagRx<'static, Async>,
    mut tx: UsbSerialJtagTx<'static, Async>,
    one_wire_pin: Flex<'static>,
    heat: Output<'static>,
    cool: Output<'static>,
) {
    let mut sensor_reader = SensorReader::new(one_wire_pin);
    let mut relay_controller = RelayController::new(heat, cool);
    let mut controller = ControllerActionRules::new(DEFAULT_TARGET_TEMP, TARGET_RANGE);
    let mut fermenter_readings = TemperatureReadings::new(FERMENTER_EMA_WINDOW);
    let mut ambient_readings = TemperatureReadings::new(AMBIENT_EMA_WINDOW);
    // REST is the Arduino's own startup default (`Action currentAction = REST`).
    let mut current_action = Action::Rest;

    // Seed both EMAs from the first sensor read, same as the Arduino's
    // setup(). If it fails, both trackers just start from TemperatureReadings'
    // own zeroed default and build up from the first successful tick instead.
    match sensor_reader.read() {
        Ok((fermenter, ambient)) => {
            fermenter_readings.set_initial_average(fermenter);
            ambient_readings.set_initial_average(ambient);
        }
        #[allow(unused_variables)]
        Err(e) => {
            #[cfg(feature = "debug-log")]
            esp_println::println!("initial sensor read failed: {e:?}");
        }
    }

    let mut buf = [0u8; 64];
    let mut frame_buf: heapless::Vec<u8, 16> = heapless::Vec::new();
    let mut frame_in_progress = false;
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut ticks_since_telemetry: u32 = 0;

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
                            if let Some(new_target) = logic::parse_target_frame(&frame_buf) {
                                controller.set_target_temp(new_target);
                                #[cfg(feature = "debug-log")]
                                esp_println::println!("target updated: {new_target}");
                            }
                            frame_buf.clear();
                            frame_in_progress = false;
                        }
                    }
                }
            }
            #[allow(unreachable_patterns)]
            Either::First(Err(_e)) => {
                #[cfg(feature = "debug-log")]
                esp_println::println!("rx read error: {:?}", _e);
            }
            Either::Second(()) => match sensor_reader.read() {
                Ok((fermenter, ambient)) => {
                    fermenter_readings.update(fermenter);
                    ambient_readings.update(ambient);

                    let decision = controller.make_action_decision(
                        current_action,
                        ambient_readings.average(),
                        fermenter_readings.average(),
                    );
                    current_action = decision.action();
                    relay_controller.set(current_action);

                    ticks_since_telemetry += 1;
                    if ticks_since_telemetry >= TELEMETRY_EVERY_N_TICKS {
                        ticks_since_telemetry = 0;
                        let telemetry = logic::format_telemetry(&TelemetryData {
                            target: controller.get_target_temp(),
                            average: fermenter_readings.average(),
                            min: fermenter_readings.minimum(),
                            max: fermenter_readings.maximum(),
                            ambient: ambient_readings.average(),
                            action: decision.action_text(),
                            reason_code: decision.reason_code(),
                        });
                        let _ = tx.write_all(telemetry.as_bytes()).await;
                        let _ = tx.flush().await;
                    }
                }
                // design.md Decision 10: a sensor read failure skips
                // make_action_decision entirely for this tick and drives the
                // fail-safe Error action directly, rather than deciding from
                // a bogus reading.
                #[allow(unused_variables)]
                Err(e) => {
                    #[cfg(feature = "debug-log")]
                    esp_println::println!("sensor read error: {e:?}");
                    current_action = Action::Error;
                    relay_controller.set(Action::Error);
                }
            },
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

    let one_wire_pin = Flex::new(peripherals.GPIO4);
    let heat = Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default());
    let cool = Output::new(peripherals.GPIO1, Level::Low, OutputConfig::default());

    spawner.spawn(control_task(rx, tx, one_wire_pin, heat, cool).unwrap());
}

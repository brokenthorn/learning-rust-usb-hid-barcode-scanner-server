use crate::constants::{INTERMEC_VID, SG20_PID};
use rusb::LogLevel;
use rusb::{Context, Device, UsbContext};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

struct HotPlugHandler;

impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        info!("Device connected: {:?}", device);
    }

    fn device_left(&mut self, device: Device<T>) {
        info!("Device disconnected: {:?}", device);
    }
}

// USB HID POS: (in Honeywell user guides referenced as USB HID / USB HID Bar Code Scanner)
//
// The USB HID POS (131) interface also referred to as HID COM, is the most underestimated
// and less used interface.
//
// Technically it is familiar to HID keyboard but then without keyboard overhead which makes it much faster.
// Today only a few software applications actually use this interface.
// USB HID POS conforms to the USB standard document "HID Point of Sales Usage Tables" V1.02 HID POS
// is the official USB method for connecting a bar code reader:
// https://honeywellsps.my.salesforce.com/sfc/p/00000000SK3U/a/A00000000FNw/srkiB9jggTCjK25Fza0x2cXvIu_OtkSif6_vGebuBzA
//

pub mod constants;
pub mod device;
pub mod server;

use crate::device::usb::UsbDeviceHandle;
use device::*;

#[tracing::instrument]
fn main() -> rusb::Result<()> {
    // Set default logging level(s):
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }

    // A builder for `FmtSubscriber`:
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    // Will be used as a fallback if no thread-local subscriber
    // has been set in a thread (using with_default):
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    rusb::set_log_level(LogLevel::Info);

    info!("Starting up.");

    //    info!("Listing devices.");
    //    list_devices().unwrap();

    //    match Context::new() {
    //        Ok(mut context) => match open_device(&mut context, INTERMEC_VID, SG20_PID) {
    //            Some((mut device, device_desc, mut handle)) => {
    //                read_device(&mut device, &device_desc, &mut handle).unwrap()
    //            }
    //            None => println!(
    //                "could not find device {:04x}:{:04x}",
    //                INTERMEC_VID, SG20_PID
    //            ),
    //        },
    //        Err(e) => panic!("could not initialize libusb: {}", e),
    //    }

    match Context::new() {
        Ok(mut context) => {
            match device::usb::open_usb_device(&mut context, INTERMEC_VID, SG20_PID) {
                None => {
                    error!("Failed to open device.");
                }
                Some(d) => {
                    let speed = d.device.speed();
                    info!("Successfully opened device: {:?}, Device Speed = {}", d, device::usb::speed_as_str(&speed));
                }
            }
        }
        Err(e) => panic!("Could not initialize libusb: {}", e),
    }

    Ok(())

    //    if rusb::has_hotplug() {
    //        let context = Context::new()?;
    //        context.register_callback(None, None, None, Box::new(HotPlugHandler {}))?;
    //
    //        loop {
    //            context.handle_events(None).unwrap();
    //        }
    //    } else {
    //        eprint!("libusb hotplug api unsupported");
    //        Ok(())
    //    }

    //    info!("Has hotplug capability: {}", rusb::has_hotplug());
    //
    //    for device in rusb::devices().unwrap().iter() {
    //        let device_desc = device.device_descriptor().unwrap();
    //
    //        let (vendor_id, product_id) = (device_desc.vendor_id(), device_desc.product_id());
    //        let (bus_number, address, port) =
    //            (device.bus_number(), device.address(), device.port_number());
    //
    //        if vendor_id == 0x067e {
    //            info!(
    //                "Found an Intermec device: [VEN: {:04x} ({: >5}), PID: {:04x} ({: >5}) at Bus: {}, Address: {}, Port: {}].",
    //                vendor_id, vendor_id, product_id, product_id, bus_number, address, port
    //            );
    //
    //            let d_handle_result = device.open();
    //
    //            if let Ok(handle) = d_handle_result {
    //                info!("Successfully opened device: [{:?}]", device);
    //
    //                let supported_languages = handle.read_languages(Duration::from_secs(10)).unwrap();
    //                info!("Supported languages: {:?}", supported_languages);
    //
    //                let device_descriptor = handle.device().device_descriptor().unwrap();
    //
    //                let manufacturer_string = handle.read_manufacturer_string(
    //                    supported_languages.first().unwrap().clone(),
    //                    &device_descriptor,
    //                    Duration::from_secs(10),
    //                );
    //
    //                let product_string = handle.read_product_string(
    //                    supported_languages.first().unwrap().clone(),
    //                    &device_descriptor,
    //                    Duration::from_secs(10),
    //                );
    //
    //                info!(
    //                    "Device: [{}, {}]",
    //                    manufacturer_string.unwrap(),
    //                    product_string.unwrap()
    //                );
    //
    //                handle.
    //            } else {
    //                warn!("Failed to open device.");
    //            }
    //        } else {
    //            info!(
    //                "Skipping non-Intermec device: [VEN: {:04x} ({: >5}), PID: {:04x} ({: >5}) at Bus: {}, Address: {}, Port: {}].",
    //                vendor_id, vendor_id, product_id, product_id, bus_number, address, port
    //            );
    //        }
    //    }
}

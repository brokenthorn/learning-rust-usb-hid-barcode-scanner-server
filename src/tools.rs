use hidapi::{HidApi, HidDevice};
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::devices::UsbDeviceIdentifier;

/// Initializes the global logging facility.
///
/// If `RUST_LOG` is not set, this function will set the global default logging level to `info`,
/// and for `server_usb` it will set the `trace` logging level.
///
/// Log messages are formatted and printed to standard output by `tracing_subscriber::FmtSubscriber`.
///
/// # Panics
///
/// Panics if the initialization was unsuccessful, likely because a global subscriber was already
/// installed by another call to try_init.
pub fn initialize_logging(json_output: bool) {
    // set default logging levels:
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info,server_usb=trace");
    }
    let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE);
    if json_output {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}

/// Refreshes hidapi's internal list of attached devices and returns that list.
#[tracing::instrument(skip(hidapi))]
pub fn refresh_devices(hidapi: &mut HidApi) -> Vec<UsbDeviceIdentifier> {
    info!("Refreshing devices list.");

    let _r = hidapi.refresh_devices();
    let mut connected_devices = vec![];

    for device in hidapi.devices() {
        if device.serial_number.is_none() {
            let device_id = UsbDeviceIdentifier::VidPid {
                vid: device.vendor_id,
                pid: device.product_id,
            };

            debug!("Found a device: {}", device_id);

            connected_devices.push(device_id);
        } else {
            let device_id = UsbDeviceIdentifier::VidPidSn {
                vid: device.vendor_id,
                pid: device.product_id,
                sn: device.serial_number.as_deref().unwrap_or(""),
            };

            debug!("Found a device: {}", device_id);

            connected_devices.push(device_id);
        }
    }

    connected_devices
}

/// Get a formatted string composed of manufacturer string and product string.
pub fn get_product_name(device: &HidDevice) -> String {
    format!(
        "{} {}",
        match device.get_manufacturer_string() {
            Ok(m) => {
                m.unwrap_or_else(|| "NA".into())
            }
            Err(e) => {
                format!("{:?}", e)
            }
        },
        match device.get_product_string() {
            Ok(m) => {
                m.unwrap_or_else(|| "NA".into())
            }
            Err(e) => {
                format!("{:?}", e)
            }
        },
    )
}

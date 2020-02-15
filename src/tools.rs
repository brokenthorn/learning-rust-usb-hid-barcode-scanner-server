use hidapi::{HidApi, HidDevice};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::constants::SUPPORTED_DEVICES;
use crate::devices::SDevice;

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

/// Check if any of the connected devices are ones that we support.
#[tracing::instrument(skip(hidapi))]
pub fn find_supported_devices(hidapi: &mut HidApi) -> Vec<SDevice> {
    info!("Refreshing devices list and searching for supported devices...");
    let _r = hidapi.refresh_devices();
    let mut supported_devices = vec![];
    for device in hidapi.devices() {
        let s_device: SDevice = SDevice {
            vid: device.vendor_id,
            pid: device.product_id,
            sn: device.serial_number.as_ref().map_or("", |sn| sn.as_str()),
        };
        if SUPPORTED_DEVICES.contains(&s_device) {
            supported_devices.push(s_device);
        }
    }
    info!("Connected supported devices: {:?}", supported_devices);
    supported_devices
}

/// Get a formatted string composed of manufacturer string and product string.
pub fn get_full_device_name(device: &HidDevice) -> String {
    format!(
        "{} {}",
        match device.get_manufacturer_string() {
            Ok(m) => {
                m.unwrap_or("NA".to_string())
            }
            Err(e) => {
                format!("{:?}", e)
            }
        },
        match device.get_product_string() {
            Ok(m) => {
                m.unwrap_or("NA".to_string())
            }
            Err(e) => {
                format!("{:?}", e)
            }
        },
    )
}
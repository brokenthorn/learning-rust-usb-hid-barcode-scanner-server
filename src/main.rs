#[macro_use]
extern crate counted_array;
extern crate derive_more;

use std::thread::sleep;
use std::time::Duration;

use hidapi::{HidApi, HidResult};
use tracing::{debug, info};

use crate::constants::INTERMEC_SG20;
use crate::server::connect_to_device;
use crate::tools::find_supported_devices;

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
// https://jpgraph.net/download/manuals/chunkhtml/ch26.html
//
// The maximum capacity for Data Matrix codes is up to 3116 numeric characters or up to 2335
// alphanumeric characters or up to 1555 bytes of binary information.
//
// The exact number of characters that can fit in a Data Matrix symbol depends on the actual encoding
// (or compaction) schema used. In short this is used to more efficiently encode ASCII characters
// to fit more data into a fixed number of bytes. For example if only numeric data is to be encoded
// then instead of using one byte to hold each digit two digits is stored in a single byte
// hence doubling the amount of data that can be stored in a given number of bytes.

pub mod constants;
pub mod decoder;
pub mod devices;
pub mod server;
pub mod tools;

/// Initializes the hidapi.
/// Will also initialize the currently available device list.
#[tracing::instrument]
pub fn initialize_hidapi() -> HidResult<HidApi> {
    debug!("Initializing the hidapi.");
    hidapi::HidApi::new()
}

#[tracing::instrument]
fn main() {
    tools::initialize_logging(false);
    info!("Starting Server USB.");

    loop {
        let hidapi_init_result = initialize_hidapi();

        match hidapi_init_result {
            Ok(mut hidapi) => {
                let _supported_devices = find_supported_devices(&mut hidapi);

                connect_to_device(INTERMEC_SG20, &mut hidapi);
            }
            Err(e) => {
                info!(
                    "Failed to initialize hidapi: {:?}. Retrying in 5 seconds...",
                    e
                );

                sleep(Duration::from_secs(5));
                continue;
            }
        }
    }
}

use std::time::Duration;

use crate::constants::{PID_SG20, VID_INTERMEC};
use crate::devices::UsbDeviceIdentifier;
use crate::server::UsbHidPosDeviceServer;

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

#[tracing::instrument]
fn main() {
    tools::initialize_logging(false);

    let device_id = UsbDeviceIdentifier::VidPidSn {
        vid: VID_INTERMEC,
        pid: PID_SG20,
        sn: "3203-11211268246",
    };

    let server = UsbHidPosDeviceServer::new(device_id);

    server.start(Duration::from_secs(5));
}

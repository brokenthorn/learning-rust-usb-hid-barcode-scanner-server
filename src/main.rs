use crate::constants::{INTERMEC_VID, SG20_PID};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

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
//pub mod device;
//pub mod server;

#[tracing::instrument]
fn main() {
    // Set default logging level(s):
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }

    FmtSubscriber::builder()
        //.compact()
        //.json()
        .init();

    info!("Starting up.");

    let hid = hid::init().unwrap();

    for device in hid.devices() {
        print!("{} ", device.path().to_str().unwrap());
        print!("ID {:x}:{:x} ", device.vendor_id(), device.product_id());

        if let Some(name) = device.manufacturer_string() {
            print!("{} ", name);
        }

        if let Some(name) = device.product_string() {
            print!("{} ", name);
        }

        if let Some(name) = device.serial_number() {
            print!("{} ", name);
        }

        println!();
    }
}

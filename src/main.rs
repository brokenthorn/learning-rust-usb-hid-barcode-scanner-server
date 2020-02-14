use crate::constants::{INTERMEC_VID, SG20_PID};
use tracing::info;
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

    loop {
        info!("Initializing HIDAPI.");

        let api_res = hidapi::HidApi::new();

        if let Err(e) = api_res {
            info!(
                "Failed to initialize HIDAPI: {:?}. Retrying in 10 seconds.",
                e
            );
            std::thread::sleep(std::time::Duration::from_secs(10));
            continue;
        }

        let mut api = api_res.unwrap();

        // Print out information about all connected devices
        //        info!("Listing devices.");
        //        for device in api.devices() {
        //            info!("{:?}", device);
        //        }

        // Connect to device using its vid and pid
        let (vid, pid) = (INTERMEC_VID, SG20_PID);

        loop {
            info!(
                "Trying to connect to device VID={:#04X?}, PID={:#04X?} (Intermec SG20)...",
                vid, pid
            );
            let _res = api.refresh_devices();
            let device_res = api.open(vid, pid);

            match device_res {
                Ok(device) => {
                    let serial_number_res = device.get_serial_number_string();
                    info!(
                        "Connected to device (SN:{:?})",
                        serial_number_res.unwrap_or_default()
                    );

                    // Read data from device
                    const BUFFER_SIZE: usize = 64;
                    info!("Using a read buffer size of {} bytes.", BUFFER_SIZE);
                    let mut buf = [0u8; BUFFER_SIZE];

                    let mut count: usize = 0;

                    loop {
                        info!("Waiting for read...");
                        let hid_res = device.read(&mut buf[..]);
                        match hid_res {
                            Ok(res) => {
                                info!("Read bytes: {:?}", &buf[..res]);
                                info!(
                                    "Lossy conversion to UTF8: {:?}",
                                    String::from_utf8_lossy(&buf[..res])
                                );

                                match &buf[res - 3..res] {
                                    [0, 40, 1] => {
                                        info!("READING NEXT BYTES...");
                                        count += *&buf[..res].len();
                                        info!("Count={}", count);
                                    }
                                    [0, 40, 0] => {
                                        info!("REPORT FINISHED. RECEIVED ALL BYTES.");
                                        info!("Count={}", count);
                                        count = 0;
                                    }
                                    [0, 1, 0] => {
                                        info!("REPORT FINISHED. RECEIVED ALL BYTES.");
                                        count += *&buf[..res].len();
                                        info!("Count={}", count);
                                        count = 0;
                                    }
                                    other => {
                                        info!("Unknown termination bytes: {:?}", other);
                                        count += *&buf[..res].len();
                                        info!("Count={}", count);
                                    }
                                }
                            }
                            Err(e) => {
                                info!("Read error: {:?}. Disconnecting.", e);
                                break;
                            }
                        }
                    }

                    // Write data to device
                    //            let buf = [0u8, 1, 2, 3, 4];
                    //            let hid_res = device.write(&buf);
                    //            match hid_res {
                    //                Ok(res) => {
                    //                    info!("Wrote: {:?} byte(s)", res);
                    //                }
                    //                Err(e) => {
                    //                    info!("Write error: {:?}", e);
                    //                }
                    //            }
                }
                Err(e) => {
                    info!("Device open error: {:?}. Reconnecting in 3 seconds.", e);
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
            }
        }
    }
}

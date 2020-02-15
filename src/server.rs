use hidapi::HidApi;
use tracing::{debug, info};

use crate::decoder::decode_aim_symbology;
use crate::devices::SDevice;
use crate::tools::get_full_device_name;

#[tracing::instrument(skip(hidapi))]
pub fn connect_to_device(s_device: SDevice, hidapi: &mut HidApi) {
    let (vid, pid, sn) = (s_device.vid, s_device.pid, s_device.sn);

    loop {
        info!("Trying to connect to {:?}", s_device);

        let _res = hidapi.refresh_devices();
        let device_res = hidapi.open_serial(vid, pid, sn);

        match device_res {
            Ok(device) => {
                let full_device_name = get_full_device_name(&device);
                info!("Connected to {:?}: {}", s_device, full_device_name);

                // Read data from device
                const BUFFER_SIZE: usize = 64 * 4; // TODO: Replace buffer size with max packet size from HID descriptor.
                info!("Read buffer size is {} bytes.", BUFFER_SIZE);

                // buffer for reading bytes from the HID device:
                let mut buf = [0u8; BUFFER_SIZE];
                // the number of bytes read so far from the current HID report:
                let mut count_bytes_read_from_current_report: usize = 0;

                loop {
                    info!("Waiting for device to send data...");
                    let read_result = device.read(&mut buf[..]);

                    match read_result {
                        Ok(bytes_read_size) => {
                            // the bytes that were read:
                            let bytes = &buf[..bytes_read_size];
                            debug!("Received (HEX): {:02X?}", &bytes);

                            let terminator = {
                                let mut t = [0u8; 3];
                                t.copy_from_slice(&bytes[(bytes_read_size - 3)..bytes_read_size]);
                                t
                            };
                            debug!("Terminator: {:02X?}", terminator);

                            let symbology = {
                                let mut s = [0u8; 3];
                                s.copy_from_slice(&bytes[2..=4]);
                                s
                            };
                            debug!(
                                "Symbology {:02X?}={:?} is {:?}.",
                                symbology,
                                String::from_utf8_lossy(&symbology),
                                decode_aim_symbology(&symbology),
                            );

                            match terminator {
                                [0, 40, 1] => {
                                    count_bytes_read_from_current_report +=
                                        *&bytes[..bytes_read_size].len();
                                    debug!(
                                        "{} bytes read so far.",
                                        count_bytes_read_from_current_report
                                    );
                                }
                                [0, 40, 0] => {
                                    info!("Finished reading HID report.");
                                    info!(
                                        "{} bytes read in total.",
                                        count_bytes_read_from_current_report
                                    );
                                    count_bytes_read_from_current_report = 0;
                                }
                                [0, 1, 0] => {
                                    info!("Finished reading HID report.");
                                    count_bytes_read_from_current_report +=
                                        *&bytes[..bytes_read_size].len();
                                    info!(
                                        "{} bytes read in total.",
                                        count_bytes_read_from_current_report
                                    );
                                    count_bytes_read_from_current_report = 0;
                                }
                                other => {
                                    info!("Unknown termination bytes: {:?}={:02X?}", other, other);
                                    count_bytes_read_from_current_report +=
                                        *&bytes[..bytes_read_size].len();
                                    info!(
                                        "{} bytes read (from unknown report size).",
                                        count_bytes_read_from_current_report
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            info!("Error reading data from device: {:?}. Disconnecting and connecting back in 3 seconds.", e);
                            std::thread::sleep(std::time::Duration::from_secs(3));
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
                info!("Device open error: {:?}. Reconnecting in 5 seconds.", e);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    }
}

use std::thread::sleep;
use std::time::Duration;

use hidapi::{HidApi, HidResult};
use tracing::{debug, info};

use crate::devices::UsbDeviceIdentifier;
use crate::tools::{get_product_name, refresh_devices};

static JAVA_ZIP_HEADER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
static BZIP2_HEADER: [u8; 10] = [0x42, 0x5A, 0x68, 0x39, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59];

/// Initializes the hidapi.
/// Will also initialize the currently available device list.
#[tracing::instrument]
pub fn initialize_hidapi() -> HidResult<HidApi> {
    debug!("Initializing the hidapi.");
    hidapi::HidApi::new()
}

#[derive(Debug)]
pub struct UsbHidPosDeviceServer<'a> {
    device_identifier: UsbDeviceIdentifier<'a>,
}

impl<'a> UsbHidPosDeviceServer<'a> {
    #[tracing::instrument()]
    pub fn new(device_identifier: UsbDeviceIdentifier<'a>) -> Self {
        info!(
            "Creating new USB HID POS Device Server for device: {}",
            device_identifier
        );

        UsbHidPosDeviceServer { device_identifier }
    }

    #[tracing::instrument()]
    pub fn start(&self, timeout: Duration) {
        info!("Starting server.");

        loop {
            let hidapi_init_result = initialize_hidapi();

            match hidapi_init_result {
                Ok(mut hidapi) => loop {
                    let present_devices = refresh_devices(&mut hidapi);

                    if present_devices.contains(&self.device_identifier) {
                        info!("Device is present: {}", self.device_identifier);

                        let device_response = {
                            match self.device_identifier {
                                UsbDeviceIdentifier::VidPid { vid, pid } => hidapi.open(vid, pid),
                                UsbDeviceIdentifier::VidPidSn { vid, pid, sn } => {
                                    hidapi.open_serial(vid, pid, sn)
                                }
                            }
                        };

                        match device_response {
                            Ok(device) => {
                                info!("Connected to {}", self.device_identifier);

                                let product_name = get_product_name(&device);

                                info!("Device name: {}.", product_name);

                                const BUFFER_SIZE: usize = 64 * 4;
                                let mut buf = [0u8; BUFFER_SIZE];
                                let mut _data_buf: Vec<u8>;

                                info!("Entering read loop.");

                                let mut num_read_errors = 0;

                                loop {
                                    info!("Waiting for read...");

                                    let read_result = device.read(&mut buf);

                                    match read_result {
                                        Ok(read_len) => {
                                            num_read_errors = 0;

                                            let bytes = &buf[..read_len];
                                            let _symbology_bytes = {
                                                let mut sym = [0u8; 3];
                                                sym.copy_from_slice(&bytes[2..=4]);
                                                sym
                                            };
                                            let _terminator_bytes = {
                                                let mut term = [0u8; 3];
                                                term.copy_from_slice(&bytes[(read_len - 3)..]);
                                                term
                                            };

                                            debug!(
                                                "Received {} bytes: {:02x?}",
                                                bytes.len(),
                                                bytes
                                            );
                                        }
                                        Err(e) => {
                                            info!("Error reading data: {:?}", e);

                                            num_read_errors += 1;

                                            if num_read_errors >= 3 {
                                                debug!("Failed to read from device 3 times in a row. Closing this device handle.");
                                                break;
                                            } else {
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                info!("Error connecting to device: {:?}", e);
                            }
                        }
                    } else {
                        info!("Device {} not connected.", self.device_identifier);
                    }

                    info!("Retrying in {:?}.", timeout);
                    sleep(timeout);
                },
                Err(e) => {
                    info!("Failed to initialize hidapi: {:?}.", e);
                }
            }

            info!("Retrying in {:?}.", timeout);
            sleep(timeout);

            continue;
        }
    }
}

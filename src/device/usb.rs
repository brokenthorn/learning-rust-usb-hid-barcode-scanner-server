use rusb::{
    Device, DeviceDescriptor, DeviceHandle, Direction, Language, Speed, SyncType, TransferType,
    UsageType, UsbContext,
};
use std::fmt::{Display, Error, Formatter, Debug};
use std::time::Duration;
use tracing::{error, info};

fn usb_context_fmt<T: UsbContext>(
    t: &T,
    f: &mut std::fmt::Formatter,
) -> Result<(), std::fmt::Error> {
    write!(f, "UsbContext")
}

/// A USB device.
pub struct UsbDevice<T: UsbContext> {
    handle: DeviceHandle<T>,
    language: Language,
    timeout: Duration,
}

/// A handle to an opened USB device.
pub struct UsbDeviceHandle<T: UsbContext> {
    pub device: Device<T>,
    pub device_desc: DeviceDescriptor,
    pub handle: DeviceHandle<T>,
}

impl<T: UsbContext> Debug for UsbDeviceHandle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "UsbDeviceHandle {{ device_desc: {:?} }}", self.device_desc)
    }
}

/// A USB device endpoint.
#[derive(Debug)]
pub struct Endpoint {
    /// The endpoint's configuration descriptor.
    config: u8,
    iface: u8,
    /// The endpoint number.
    number: u8,
    setting: u8,
    /// The endpoint's address.
    address: u8,
    /// The endpoint's direction.
    direction: Direction,
    /// The endpoint's transfer type.
    transfer_type: TransferType,
    /// The endpoint's synchronisation mode. This is only valid for isochronous endpoints.
    sync_type: SyncType,
    /// The endpoint's usage type. This is only valid for isochronous endpoints.
    usage_type: UsageType,
    /// The endpoint's maximum packet size.
    max_packet_size: u16,
    /// The endpoint's polling interval.
    b_interval: u8,
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Endpoint Address {:#04x} EP {} {:?}",
            self.address, self.number, self.direction
        )
    }
}

/// Convert to USB speed standard to human readable value.
pub fn speed_as_str(speed: &Speed) -> &'static str {
    match speed {
        Speed::Unknown => "5000 Mbps",
        Speed::Low => "480 Mbps",
        Speed::Full => "12 Mbps",
        Speed::High => "1.5 Mbps",
        Speed::Super => "(unknown)",
    }
}

/// Open a USB device and get back a device handle.
pub fn open_usb_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<UsbDeviceHandle<T>> {
    info!(
        "Looking to open an USB device with VID={:#04x} & PID={:#04x}...",
        vid, pid
    );
    let mut at_least_one_was_found = false;

    let devices = match context.devices() {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to get a list of current USB devices: {}", e);
            return None;
        }
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(e) => {
                info!("Skipping a device because we failed to get the device descriptor.");
                error!("Failed to get device descriptor: {}", e);
                continue;
            }
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            info!(
                "Found a matching device with VID={:#04x} & PID={:#04x}. Opening it now.",
                vid, pid
            );

            at_least_one_was_found = true;

            match device.open() {
                Ok(handle) => {
                    return Some(UsbDeviceHandle {
                        device,
                        device_desc,
                        handle,
                    })
                }
                Err(e) => {
                    info!("Skipping the device because we failed to open it.");
                    error!("Failed to open the device: {}", e);
                    continue;
                }
            }
        }
    }

    if at_least_one_was_found {
        info!("Couldn't open any of the matching devices found.");
    } else {
        info!("Didn't find any matching devices to open.");
    }

    None
}

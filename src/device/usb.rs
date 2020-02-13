use rusb::{
    Device, DeviceDescriptor, DeviceHandle, Direction, Language, Speed, SyncType, TransferType,
    UsageType, UsbContext,
};
use std::fmt::{Debug, Display, Error, Formatter};
use std::slice;
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
        write!(
            f,
            "UsbDeviceHandle {{ device_desc: {:?} }}",
            self.device_desc
        )
    }
}

/// A USB device endpoint.
#[derive(Debug)]
pub struct Endpoint {
    /// The endpoint's configuration descriptor.
    config: u8,
    /// The interface's number.
    iface: u8,
    /// The alternate setting number.
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
            // TODO: Add more info:
            "Endpoint Address {:#04x}, Direction{:?}",
            self.address, self.direction
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

/// Finds a readable endpoint of a specified transfer type.
pub fn find_readable_endpoint<T: UsbContext>(
    usb_device_handle: UsbDeviceHandle<T>,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    info!(
        "Looking for the first readable endpoint with transfer type {:?}.",
        transfer_type
    );

    // iterate over all configurations, pick the first one that's readable:
    for n in 0..usb_device_handle.device_desc.num_configurations() {
        let config_desc = match usb_device_handle.device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // find the first endpoint that's Direction::In and the requested transfer type:
        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == Direction::In
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                            direction: endpoint_desc.direction(),
                            transfer_type: endpoint_desc.transfer_type(),
                            sync_type: endpoint_desc.sync_type(),
                            usage_type: endpoint_desc.usage_type(),
                            max_packet_size: endpoint_desc.max_packet_size(),
                            b_interval: endpoint_desc.interval(),
                        });
                    }
                }
            }
        }
    }

    info!(
        "No readable endpoint found with transfer type {:?}.",
        transfer_type
    );
    None
}

/// Open a USB device and get back a device handle.
pub fn open_usb_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<UsbDeviceHandle<T>> {
    info!(
        "Looking to open a USB device with VID={:#04x} & PID={:#04x}...",
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

/// Activate an endpoint and claim it for use.
pub fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> rusb::Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

pub fn pool_interrupt_endpoint<T: UsbContext>(
    usb_device_handle: &mut UsbDeviceHandle<T>,
    endpoint: self::Endpoint,
) {
    info!("Started pooling Interrupt transfer type Endpoint.");

    let has_kernel_driver = match usb_device_handle
        .handle
        .kernel_driver_active(endpoint.iface)
    {
        Ok(true) => {
            info!("The endpoint has a kernel-mode driver which needs to be detached for us to be able to connect to it from user-space.");
            let res = usb_device_handle
                .handle
                .detach_kernel_driver(endpoint.iface);
            if res.is_err() {
                error!(
                    "Failed to detach kernel driver from endpoint: {:?}",
                    endpoint
                );
                return;
            }
            true
        }
        _ => false,
    };

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut vec = Vec::<u8>::with_capacity(256);
            let buf =
                unsafe { slice::from_raw_parts_mut((&mut vec[..]).as_mut_ptr(), vec.capacity()) };

            let timeout = Duration::from_secs(1);

            match transfer_type {
                TransferType::Interrupt => {
                    let mut i: u64 = 0;
                    loop {
                        match handle.read_interrupt(endpoint.address, buf, timeout) {
                            Ok(len) => {
                                unsafe { vec.set_len(len) };
                                println!("{: >6}: - read {} bytes: {:?}", i, len, vec);
                                println!("      : {:?}", String::from_utf8_lossy(&vec));
                                i += 1;
                            }
                            Err(_err) => {}
                        }
                    }
                }
                TransferType::Bulk => {
                    loop {
                        match handle.read_bulk(endpoint.address, buf, timeout) {
                            Ok(len) => {
                                unsafe { vec.set_len(len) };
                                println!(" - read: {:?}", vec);
                                // println!(" - read: {:?}", String::from_utf8(vec.clone()));
                            }
                            Err(_err) => {}
                        }
                    }
                }
                _ => (),
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        info!("Reattaching kernel driver to endpoint: {:?}.", endpoint);
        usb_device_handle
            .handle
            .attach_kernel_driver(endpoint.iface)
            .ok();
    }
}

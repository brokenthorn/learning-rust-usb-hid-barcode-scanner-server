use rusb::LogLevel;
use rusb::{
    ConfigDescriptor, Context, Device, DeviceDescriptor, DeviceHandle, DeviceList, Direction,
    EndpointDescriptor, InterfaceDescriptor, Language, Result, Speed, TransferType, UsbContext,
};
use std::slice;
use std::time::Duration;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

struct HotPlugHandler;

impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        info!("Device connected: {:?}", device);
    }

    fn device_left(&mut self, device: Device<T>) {
        info!("Device disconnected: {:?}", device);
    }
}

struct UsbDevice<T: UsbContext> {
    handle: DeviceHandle<T>,
    language: Language,
    timeout: Duration,
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn read_device<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    handle.reset()?;

    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    println!("Languages: {:?}", languages);

    if languages.len() > 0 {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle
                .read_product_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, device_desc, timeout)
                .ok()
        );
    }

    match find_readable_endpoint(device, device_desc, TransferType::Interrupt) {
        Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Interrupt),
        None => println!("No readable interrupt endpoint"),
    }

    match find_readable_endpoint(device, device_desc, TransferType::Bulk) {
        Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Bulk),
        None => println!("No readable bulk endpoint"),
    }

    Ok(())
}

fn find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

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
                        });
                    }
                }
            }
        }
    }

    None
}

fn read_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    transfer_type: TransferType,
) {
    println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        }
        _ => false,
    };

    println!(" - kernel driver? {}", has_kernel_driver);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut vec = Vec::<u8>::with_capacity(256);
            let buf =
                unsafe { slice::from_raw_parts_mut((&mut vec[..]).as_mut_ptr(), vec.capacity()) };

            let timeout = Duration::from_secs(1);

            match transfer_type {
                TransferType::Interrupt => {
                    match handle.read_interrupt(endpoint.address, buf, timeout) {
                        Ok(len) => {
                            unsafe { vec.set_len(len) };
                            println!(" - read: {:?}", vec);
                        }
                        Err(err) => println!("could not read from endpoint: {}", err),
                    }
                }
                TransferType::Bulk => match handle.read_bulk(endpoint.address, buf, timeout) {
                    Ok(len) => {
                        unsafe { vec.set_len(len) };
                        println!(" - read: {:?}", vec);
                    }
                    Err(err) => println!("could not read from endpoint: {}", err),
                },
                _ => (),
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).ok();
    }
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

fn list_devices() -> Result<()> {
    let timeout = Duration::from_secs(1);

    for device in DeviceList::new()?.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let mut usb_device = {
            match device.open() {
                Ok(h) => match h.read_languages(timeout) {
                    Ok(l) => {
                        if l.len() > 0 {
                            Some(UsbDevice {
                                handle: h,
                                language: l[0],
                                timeout,
                            })
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
                Err(_) => None,
            }
        };

        println!(
            "Bus {:03} Device {:03} ID {:04x}:{:04x} {}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id(),
            get_speed(device.speed())
        );
        print_device(&device_desc, &mut usb_device);

        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            print_config(&config_desc, &mut usb_device);

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    print_interface(&interface_desc, &mut usb_device);

                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        print_endpoint(&endpoint_desc);
                    }
                }
            }
        }

        println!();
    }

    Ok(())
}

fn print_device<T: UsbContext>(device_desc: &DeviceDescriptor, handle: &mut Option<UsbDevice<T>>) {
    println!("Device Descriptor:");
    println!(
        "  bcdUSB             {:2}.{}{}",
        device_desc.usb_version().major(),
        device_desc.usb_version().minor(),
        device_desc.usb_version().sub_minor()
    );
    println!("  bDeviceClass        {:#04x}", device_desc.class_code());
    println!(
        "  bDeviceSubClass     {:#04x}",
        device_desc.sub_class_code()
    );
    println!("  bDeviceProtocol     {:#04x}", device_desc.protocol_code());
    println!("  bMaxPacketSize0      {:3}", device_desc.max_packet_size());
    println!("  idVendor          {:#06x}", device_desc.vendor_id());
    println!("  idProduct         {:#06x}", device_desc.product_id());
    println!(
        "  bcdDevice          {:2}.{}{}",
        device_desc.device_version().major(),
        device_desc.device_version().minor(),
        device_desc.device_version().sub_minor()
    );
    println!(
        "  iManufacturer        {:3} {}",
        device_desc.manufacturer_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_manufacturer_string(h.language, device_desc, h.timeout)
            .unwrap_or(String::new()))
    );
    println!(
        "  iProduct             {:3} {}",
        device_desc.product_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_product_string(h.language, device_desc, h.timeout)
            .unwrap_or(String::new()))
    );
    println!(
        "  iSerialNumber        {:3} {}",
        device_desc.serial_number_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_serial_number_string(h.language, device_desc, h.timeout)
            .unwrap_or(String::new()))
    );
    println!(
        "  bNumConfigurations   {:3}",
        device_desc.num_configurations()
    );
}

fn print_config<T: UsbContext>(config_desc: &ConfigDescriptor, handle: &mut Option<UsbDevice<T>>) {
    println!("  Config Descriptor:");
    println!(
        "    bNumInterfaces       {:3}",
        config_desc.num_interfaces()
    );
    println!("    bConfigurationValue  {:3}", config_desc.number());
    println!(
        "    iConfiguration       {:3} {}",
        config_desc.description_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_configuration_string(h.language, config_desc, h.timeout)
            .unwrap_or(String::new()))
    );
    println!("    bmAttributes:");
    println!("      Self Powered     {:>5}", config_desc.self_powered());
    println!("      Remote Wakeup    {:>5}", config_desc.remote_wakeup());
    println!("    bMaxPower           {:4}mW", config_desc.max_power());

    if let Some(extra) = config_desc.extra() {
        println!("    {:?}", extra);
    } else {
        println!("    no extra data");
    }
}

fn print_interface<T: UsbContext>(
    interface_desc: &InterfaceDescriptor,
    handle: &mut Option<UsbDevice<T>>,
) {
    println!("    Interface Descriptor:");
    println!(
        "      bInterfaceNumber     {:3}",
        interface_desc.interface_number()
    );
    println!(
        "      bAlternateSetting    {:3}",
        interface_desc.setting_number()
    );
    println!(
        "      bNumEndpoints        {:3}",
        interface_desc.num_endpoints()
    );
    println!(
        "      bInterfaceClass     {:#04x}",
        interface_desc.class_code()
    );
    println!(
        "      bInterfaceSubClass  {:#04x}",
        interface_desc.sub_class_code()
    );
    println!(
        "      bInterfaceProtocol  {:#04x}",
        interface_desc.protocol_code()
    );
    println!(
        "      iInterface           {:3} {}",
        interface_desc.description_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_interface_string(h.language, interface_desc, h.timeout)
            .unwrap_or(String::new()))
    );

    if let Some(extra) = interface_desc.extra() {
        println!("    {:?}", extra);
    } else {
        println!("    no extra data");
    }
}

fn print_endpoint(endpoint_desc: &EndpointDescriptor) {
    println!("      Endpoint Descriptor:");
    println!(
        "        bEndpointAddress    {:#04x} EP {} {:?}",
        endpoint_desc.address(),
        endpoint_desc.number(),
        endpoint_desc.direction()
    );
    println!("        bmAttributes:");
    println!(
        "          Transfer Type          {:?}",
        endpoint_desc.transfer_type()
    );
    println!(
        "          Synch Type             {:?}",
        endpoint_desc.sync_type()
    );
    println!(
        "          Usage Type             {:?}",
        endpoint_desc.usage_type()
    );
    println!(
        "        wMaxPacketSize    {:#06x}",
        endpoint_desc.max_packet_size()
    );
    println!(
        "        bInterval            {:3}",
        endpoint_desc.interval()
    );
}

fn get_speed(speed: Speed) -> &'static str {
    match speed {
        Speed::Super => "5000 Mbps",
        Speed::High => " 480 Mbps",
        Speed::Full => "  12 Mbps",
        Speed::Low => " 1.5 Mbps",
        Speed::Unknown => "(unknown)",
    }
}

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

#[tracing::instrument]
fn main() -> rusb::Result<()> {
    // Set default logging level(s):
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug");
    }

    // A builder for `FmtSubscriber`:
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    // Will be used as a fallback if no thread-local subscriber
    // has been set in a thread (using with_default):
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting up service.");

    rusb::set_log_level(LogLevel::Debug);

    info!("Listing devices.");
    list_devices().unwrap();

    let intermec_vid = 0x067e;
    let sg20_pid = 0x0809;

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, intermec_vid, sg20_pid) {
            Some((mut device, device_desc, mut handle)) => {
                read_device(&mut device, &device_desc, &mut handle).unwrap()
            }
            None => println!("could not find device {:04x}:{:04x}", intermec_vid, sg20_pid),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }

    Ok(())

    //    if rusb::has_hotplug() {
    //        let context = Context::new()?;
    //        context.register_callback(None, None, None, Box::new(HotPlugHandler {}))?;
    //
    //        loop {
    //            context.handle_events(None).unwrap();
    //        }
    //    } else {
    //        eprint!("libusb hotplug api unsupported");
    //        Ok(())
    //    }

    //    info!("Has hotplug capability: {}", rusb::has_hotplug());
    //
    //    for device in rusb::devices().unwrap().iter() {
    //        let device_desc = device.device_descriptor().unwrap();
    //
    //        let (vendor_id, product_id) = (device_desc.vendor_id(), device_desc.product_id());
    //        let (bus_number, address, port) =
    //            (device.bus_number(), device.address(), device.port_number());
    //
    //        if vendor_id == 0x067e {
    //            info!(
    //                "Found an Intermec device: [VEN: {:04x} ({: >5}), PID: {:04x} ({: >5}) at Bus: {}, Address: {}, Port: {}].",
    //                vendor_id, vendor_id, product_id, product_id, bus_number, address, port
    //            );
    //
    //            let d_handle_result = device.open();
    //
    //            if let Ok(handle) = d_handle_result {
    //                info!("Successfully opened device: [{:?}]", device);
    //
    //                let supported_languages = handle.read_languages(Duration::from_secs(10)).unwrap();
    //                info!("Supported languages: {:?}", supported_languages);
    //
    //                let device_descriptor = handle.device().device_descriptor().unwrap();
    //
    //                let manufacturer_string = handle.read_manufacturer_string(
    //                    supported_languages.first().unwrap().clone(),
    //                    &device_descriptor,
    //                    Duration::from_secs(10),
    //                );
    //
    //                let product_string = handle.read_product_string(
    //                    supported_languages.first().unwrap().clone(),
    //                    &device_descriptor,
    //                    Duration::from_secs(10),
    //                );
    //
    //                info!(
    //                    "Device: [{}, {}]",
    //                    manufacturer_string.unwrap(),
    //                    product_string.unwrap()
    //                );
    //
    //                handle.
    //            } else {
    //                warn!("Failed to open device.");
    //            }
    //        } else {
    //            info!(
    //                "Skipping non-Intermec device: [VEN: {:04x} ({: >5}), PID: {:04x} ({: >5}) at Bus: {}, Address: {}, Port: {}].",
    //                vendor_id, vendor_id, product_id, product_id, bus_number, address, port
    //            );
    //        }
    //    }
}

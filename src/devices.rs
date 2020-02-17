use std::fmt::Debug;

use derive_more::Display;

/// Identifier for USB devices supported by this application.
#[derive(Debug, Display, Eq, PartialEq, Clone, Copy)]
pub enum UsbDeviceIdentifier<'a> {
    /// A device matching a vendor ID (vid) and a product ID (pid).
    #[display(
        fmt = "VidPid {{ vid: {:04x?}, pid: {:04x?} }}",
        vid,
        pid
    )]
    VidPid { vid: u16, pid: u16 },

    /// A device matching a vendor ID (vid), a product ID (pid) and a (usually) unique product
    /// serial number (sn).
    #[display(
        fmt = "VidPidSn {{ vid: {:04x?}, pid: {:04x?}, sn: {} }}",
        vid,
        pid,
        sn
    )]
    VidPidSn { vid: u16, pid: u16, sn: &'a str },
}

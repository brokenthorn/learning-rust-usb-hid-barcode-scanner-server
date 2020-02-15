use std::fmt::{Debug, Error, Formatter};

/// A USB-HID device that ServerUSB supports.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SDevice<'a> {
    /// Vendor ID.
    pub vid: u16,
    /// Product ID.
    pub pid: u16,
    /// Device Serial Number.
    pub sn: &'a str,
}

impl<'a> Debug for SDevice<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "SDevice {{ vid: {:04X?}, pid: {:04X?}, sn: \"{}\" }}",
            self.vid, self.pid, self.sn
        )
    }
}

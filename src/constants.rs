use crate::devices::SDevice;

/// Intermec's Vendor ID.
pub static VID_INTERMEC: u16 = 0x067e;

/// Intermec SG20(T/THP) Product ID.
pub static PID_SG20: u16 = 0x0809;

/// Intermec SG20(T/THP) SDevice static instance.
pub static INTERMEC_SG20: SDevice = SDevice {
    vid: VID_INTERMEC,
    pid: PID_SG20,
    sn: "",
};

counted_array!(pub static SUPPORTED_DEVICES: [SDevice; _] = [
INTERMEC_SG20
]);

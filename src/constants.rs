/// Intermec's Vendor ID.
pub static INTERMEC_VID: u16 = 0x067e;

/// Intermec SG20 Device Product ID.
pub static SG20_PID: u16 = 0x0809;

counted_array!(
pub static SUPPORTED_VIDS: [u16; _] = [INTERMEC_VID]
);

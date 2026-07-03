mod hwid_read;
mod hwid_read_legacy;

pub use hwid_read::*;
pub use hwid_read_legacy::*;

use core::fmt;
use heapless::String;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Hwid {
    // fuse 0x01
    pub cfg0: u32,
    // fuse 0x02
    pub cfg1: u32,
    // fuse 0x22
    pub mac0: u32,
    // fuse 0x23
    pub mac1: u32,
}

impl fmt::Display for Hwid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}-{:08x}-{:08x}-{:08x}", self.cfg0, self.cfg1, self.mac0, self.mac1)
    }
}

impl TryFrom<&str> for Hwid {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.len() != 35 {
            return Err("hwid should be exactly 35 chars");
        }

        let raw: String<32> = s.chars().filter(|&c| c != '-').collect();

        let cfg0 = u32::from_str_radix(&raw[0..8], 16).map_err(|_| "failed to decode hwid string")?;
        let cfg1 = u32::from_str_radix(&raw[8..16], 16).map_err(|_| "failed to decode hwid string")?;
        let mac0 = u32::from_str_radix(&raw[16..24], 16).map_err(|_| "failed to decode hwid string")?;
        let mac1 = u32::from_str_radix(&raw[24..32], 16).map_err(|_| "failed to decode hwid string")?;

        Ok(Hwid { cfg0, cfg1, mac0, mac1 })
    }
}

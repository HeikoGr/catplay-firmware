extern crate libc;

use super::{Hwid, read_hwid};
use crate::nostd::SmallFd;

unsafe fn read_fsl_otp_word(c_path: &'static str) -> Result<u32, &'static str> {
    let fd = SmallFd::open_readonly(c_path)?;

    let mut buf = [0u8; 32];
    let n = fd.read(&mut buf)?;
    if n == 0 {
        return Err("failed to read fd to check hwid");
    }

    let s = core::str::from_utf8(&buf[..n]).map_err(|_| "failed to convert hex data")?;

    let trimmed = s.trim();
    let num_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let val = u32::from_str_radix(num_str, 16).map_err(|_| "failed to parse hex data")?;
    Ok(val)
}

pub fn read_hwid_legacy_kernel() -> Result<Hwid, &'static str> {
    unsafe {
        let cfg0 = read_fsl_otp_word("/sys/fsl_otp/HW_OCOTP_CFG0\0")?;
        let cfg1 = read_fsl_otp_word("/sys/fsl_otp/HW_OCOTP_CFG1\0")?;
        let mac0 = read_fsl_otp_word("/sys/fsl_otp/HW_OCOTP_MAC0\0")?;
        let mac1 = read_fsl_otp_word("/sys/fsl_otp/HW_OCOTP_MAC1\0")?;
        Ok(Hwid { cfg0, cfg1, mac0, mac1 })
    }
}

pub fn read_hwid_or_legacy() -> Result<Hwid, &'static str> {
    read_hwid().or_else(|_| read_hwid_legacy_kernel()).map_err(|_| "failed to read HWID")
}

impl Hwid {
    pub fn detect() -> Result<Hwid, &'static str> {
        read_hwid_or_legacy()
    }
}

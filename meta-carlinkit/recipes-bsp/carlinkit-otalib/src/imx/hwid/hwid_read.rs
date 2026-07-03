extern crate libc;

use super::Hwid;
use crate::nostd::SmallFd;

const OCOTP_MAX: usize = 1024;

fn read_ocotp() -> Result<[u32; OCOTP_MAX], &'static str> {
    let path = "/sys/bus/nvmem/devices/imx-ocotp0/nvmem\0";

    let fd = SmallFd::open(path)?;

    let mut buf = [0u8; OCOTP_MAX * 4];

    let n = fd.read(&mut buf)?;

    if n == 0 {
        return Err("failed to read fd");
    }

    let nwords = n / 4;
    if nwords > OCOTP_MAX {
        return Err("ocotp too big");
    }

    let mut words = [0u32; OCOTP_MAX];

    for i in 0..nwords {
        let b0 = buf[i * 4] as u32;
        let b1 = buf[i * 4 + 1] as u32;
        let b2 = buf[i * 4 + 2] as u32;
        let b3 = buf[i * 4 + 3] as u32;
        words[i] = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
    }

    Ok(words)
}

fn parse_hwid_from_ocotp(ocotp: &[u32]) -> Hwid {
    Hwid {
        cfg0: ocotp[0x01],
        cfg1: ocotp[0x02],
        mac0: ocotp[0x22],
        mac1: ocotp[0x23],
    }
}

pub fn read_hwid() -> Result<Hwid, &'static str> {
    let ocotp = read_ocotp()?;
    Ok(parse_hwid_from_ocotp(&ocotp))
}

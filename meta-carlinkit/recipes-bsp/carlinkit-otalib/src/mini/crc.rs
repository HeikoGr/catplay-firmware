use core::cmp;

pub const CKSUM_TABLE: [u32; 256] = make_cksum_table();

const fn make_cksum_table() -> [u32; 256] {
    const POLY: u32 = 0x04C11DB7;

    let mut tbl = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = (i as u32) << 24;
        let mut bit = 0;
        while bit < 8 {
            if (crc & 0x8000_0000) != 0 {
                crc = (crc << 1) ^ POLY;
            } else {
                crc <<= 1;
            }
            bit += 1;
        }
        tbl[i] = crc;
        i += 1;
    }
    tbl
}

pub const fn cksum_crc32(data: &[u8]) -> u32 {
    let mut crc = 0u32;
    let mut i = 0;
    let n = data.len();
    while i + 4 <= n {
        let a = data[i] as u32;
        let b = data[i + 1] as u32;
        let c = data[i + 2] as u32;
        let d = data[i + 3] as u32;

        crc = (crc << 8) ^ CKSUM_TABLE[((crc >> 24) ^ a) as usize];
        crc = (crc << 8) ^ CKSUM_TABLE[((crc >> 24) ^ b) as usize];
        crc = (crc << 8) ^ CKSUM_TABLE[((crc >> 24) ^ c) as usize];
        crc = (crc << 8) ^ CKSUM_TABLE[((crc >> 24) ^ d) as usize];
        i += 4;
    }
    while i < n {
        let idx = ((crc >> 24) ^ data[i] as u32) & 0xFF;
        crc = (crc << 8) ^ CKSUM_TABLE[idx as usize];
        i += 1;
    }

    let mut len = n as u32;
    while len != 0 {
        let idx = ((crc >> 24) ^ (len & 0xFF)) & 0xFF;
        crc = (crc << 8) ^ CKSUM_TABLE[idx as usize];
        len >>= 8;
    }
    crc
}

pub fn xor_crc_img(data: &[u8], bs_size: usize) -> u32 {
    let mut crc_total: u32 = 0;
    let mut off = 0;
    while off < data.len() {
        let end = cmp::min(off + bs_size, data.len());
        crc_total ^= cksum_crc32(&data[off..end]);
        off = end;
    }
    crc_total
}

#[test]
fn test_cksum_equiv() {
    let data = b"123456789";
    let crc = cksum_crc32(data);
    assert_eq!(crc, 0xC8859FEE);
}

#[test]
fn test_xor_blocks() {
    let data = [0u8; 260];
    let crc = xor_crc_img(&data, 128);
    assert_ne!(crc, 0);
}

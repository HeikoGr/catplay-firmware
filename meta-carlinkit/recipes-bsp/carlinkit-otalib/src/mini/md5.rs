use md5hash::MD5Hasher;

pub fn parse_md5_hex(s: &str) -> Option<[u8; 16]> {
    const fn from_hex_nibble(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }

    if s.len() != 32 {
        return None;
    }

    let mut out = [0u8; 16];
    let bytes = s.as_bytes();

    for i in 0..16 {
        let hi = from_hex_nibble(bytes[i * 2])?;
        let lo = from_hex_nibble(bytes[i * 2 + 1])?;
        out[i] = (hi << 4) | lo;
    }

    Some(out)
}

pub fn md5_calc(data: &[u8]) -> [u8; 16] {
    let mut hasher = MD5Hasher::new();
    hasher.digest(&data);
    let hash = hasher.finish();
    hash.as_ref().try_into().unwrap()
}

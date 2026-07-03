use aes::Aes128;
use cbc::Encryptor;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use crc32fast::Hasher;
use miniz_oxide::deflate::compress_to_vec;

use crate::{imx::hwid::Hwid, imx::signer::create_env};

type Aes128CbcEnc = Encryptor<Aes128>;

pub const ENV_CSF_PROVISION_BLOCK: usize = 0x20000;
pub const ENV_CSF_PROVISION_BLOCK_LEN: usize = 0x10000;
pub const ENV_ENCRYPTED_BLOCK: usize = 0x30000;
pub const ENV_ENCRYPTED_BLOCK_LEN: usize = 0x10000;

fn encrypt_in_place(data: &mut [u8], key: &[u8; 16], iv: &[u8; 16]) {
    let mut cipher = Aes128CbcEnc::new_from_slices(key, iv).unwrap();

    for chunk in data.chunks_exact_mut(16) {
        cipher.encrypt_blocks_mut(core::slice::from_mut(chunk.into()));
    }
}

fn gzip_compress(input: &[u8], out: &mut [u8]) -> Result<usize, &'static str> {
    let header: [u8; 10] = [0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03];

    let mut pos = 0;
    if out.len() < header.len() {
        return Err("buffer too small");
    }
    out[..header.len()].copy_from_slice(&header);
    pos += header.len();

    let compressed = compress_to_vec(input, 9);
    if pos + compressed.len() + 8 > out.len() {
        return Err("buffer too small");
    }
    out[pos..pos + compressed.len()].copy_from_slice(&compressed);
    pos += compressed.len();

    let crc = crc32(input);
    let len = input.len() as u32;

    out[pos..pos + 4].copy_from_slice(&crc.to_le_bytes());
    pos += 4;
    out[pos..pos + 4].copy_from_slice(&len.to_le_bytes());
    pos += 4;

    Ok(pos)
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

pub fn crc32x2(data: &[u8], data_b: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.update(data_b);
    hasher.finalize()
}

pub fn env_pack(env: &[u8], dtb: &[u8], hwid: Hwid, out: &mut [u8]) -> Result<usize, &'static str> {
    const ENV_TOTAL: usize = 0x800;
    const DTB_TOTAL: usize = 0x10000 - 0x800;
    const BLOCK_LEN: usize = 0x10000;

    if env.len() > ENV_TOTAL - 4 {
        return Err("env too big");
    }
    if dtb.len() > DTB_TOTAL - 4 {
        return Err("dtb too big");
    }

    if out.len() < BLOCK_LEN {
        return Err("output too small");
    }

    let mut env_buf = [0u8; ENV_TOTAL];
    env_buf[4..4 + env.len()].copy_from_slice(env);
    let env_crc = crc32(&env_buf[4..]);
    env_buf[0..4].copy_from_slice(&env_crc.to_le_bytes());

    let mut dtb_buf = [0u8; DTB_TOTAL - 4];
    dtb_buf[..dtb.len()].copy_from_slice(dtb);

    let crc = crc32x2(&env_buf, &dtb_buf);

    let mut raw = [0u8; BLOCK_LEN];
    raw[..ENV_TOTAL].copy_from_slice(&env_buf);
    raw[ENV_TOTAL..ENV_TOTAL + (DTB_TOTAL - 4)].copy_from_slice(&dtb_buf);
    raw[BLOCK_LEN - 4..].copy_from_slice(&crc.to_le_bytes());

    let mut gz_buf = [0u8; BLOCK_LEN];
    let mut data_len = gzip_compress(&raw, &mut gz_buf)?;

    let key = format_u32x2(hwid.cfg0, hwid.cfg1);
    let iv = format_u32x2(hwid.mac0, hwid.mac1);

    let pad_len = (16 - (data_len % 16)) % 16;
    for i in 0..pad_len {
        gz_buf[data_len + i] = 0;
    }
    data_len += pad_len;
    encrypt_in_place(&mut gz_buf[..data_len], &key, &iv);

    let offset = (data_len + 16) as u32;
    gz_buf[data_len..data_len + 12].fill(0);
    gz_buf[data_len + 12..data_len + 16].copy_from_slice(&offset.to_le_bytes());
    data_len += 16;

    if data_len > 0x10000 {
        return Err("compressed too big");
    }

    out[..data_len].copy_from_slice(&gz_buf[..data_len]);
    Ok(data_len)
}

/// Patch block between 0x20000 and 0x30000
pub fn patch_csf_env_provision_block(
    data: &mut [u8],
    uboot_text_base: u32,
    uboot_flash_offset: u32,
    uboot_flash_size: u32,
    uboot_printf_addr: u32,
    hwid: Hwid,
) -> Result<(), &'static str> {
    let patched_env = create_env::<1024>(uboot_text_base, uboot_flash_offset, uboot_flash_size, uboot_printf_addr);

    let mut packed = [0u8; ENV_CSF_PROVISION_BLOCK_LEN];
    let size = env_pack(patched_env.as_bytes(), b"", hwid, &mut packed)?;

    let s = ENV_CSF_PROVISION_BLOCK_LEN - size;
    let l = size;

    data[s..s + l].copy_from_slice(&packed[..l]);
    Ok(())
}

/// Patch block between 0x30000 and 0x40000
pub fn patch_encrypted_env_block(data: &mut [u8]) {
    let empty = [0u8; ENV_ENCRYPTED_BLOCK_LEN];
    data[0..ENV_ENCRYPTED_BLOCK_LEN].copy_from_slice(&empty);
}

fn format_u32x2(a: u32, b: u32) -> [u8; 16] {
    let mut s = [0u8; 16];
    hex_write(&mut s[0..8], a);
    hex_write(&mut s[8..16], b);
    s
}

fn hex_write(buf: &mut [u8], val: u32) {
    let hex = b"0123456789abcdef";
    (0..8).for_each(|i| {
        let shift = (28 - i * 4) as u32;
        buf[i] = hex[((val >> shift) & 0xf) as usize];
    });
}

#[cfg(test)]
mod test {
    const TEST_HWID: Hwid = Hwid {
        cfg0: 0x6920_116d,
        cfg1: 0x4736_49d7,
        mac0: 0x3878_0e11,
        mac1: 0x66c5_0a99,
    };

    use crate::imx::hwid::Hwid;
    use crate::imx::signer::env_pack;

    fn to_hex_string(data: &[u8]) -> String {
        let mut s = String::with_capacity(data.len() * 2);
        for b in data {
            use core::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        s
    }

    #[test]
    fn test_env_pack() {
        let env = b"testenv";
        let dtb = b"testdtb";

        let mut out = [0u8; 0x10000];

        let size = env_pack(env, dtb, TEST_HWID, &mut out).expect("env_pack failed");

        println!("Packed size = {size}");
        println!("Hex dump:\n{}", to_hex_string(&out[..size]));
    }
}

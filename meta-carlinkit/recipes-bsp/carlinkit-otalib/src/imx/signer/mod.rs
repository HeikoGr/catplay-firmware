mod env;
mod shellcode;

pub use env::*;
pub use shellcode::*;

use crate::imx::hwid::Hwid;

const OLD_UBOOT_PRINTF_ADDR: u32 = 0x87fc90e8;
const OLD_UBOOT_CRC: u32 = 0x26b2a736;

const UBOOT_TEXT_BASE: u32 = 0x87800000;
const UBOOT_FLASH_OFFSET: u32 = 0x40000;
const UBOOT_FLASH_SIZE: u32 = 0x80000;

pub fn calc_uboot_crc(wic_buf: &[u8]) -> u32 {
    let mut buf = [0u8; 0x20000];
    buf[0x400..0x20000].copy_from_slice(&wic_buf[0x400..0x20000]);
    crc32(&buf)
}

pub fn sign_wic(wic_buf: &mut [u8], hwid: Hwid) -> Result<(), &'static str> {
    #[cfg(feature = "signer")]
    {
        let crc = calc_uboot_crc(wic_buf);
        if crc != OLD_UBOOT_CRC {
            return Err("invalid uboot crc (unknown bootloader revision)");
        }

        patch_csf_env_provision_block(
            &mut wic_buf[ENV_CSF_PROVISION_BLOCK..ENV_CSF_PROVISION_BLOCK + ENV_CSF_PROVISION_BLOCK_LEN],
            UBOOT_TEXT_BASE,
            UBOOT_FLASH_OFFSET,
            UBOOT_FLASH_SIZE,
            OLD_UBOOT_PRINTF_ADDR,
            hwid,
        )?;
        patch_encrypted_env_block(&mut wic_buf[ENV_ENCRYPTED_BLOCK..ENV_ENCRYPTED_BLOCK + ENV_ENCRYPTED_BLOCK_LEN]);
        return Ok(());
    }

    Err("sign_wic disabled at compile time")
}

#[cfg(test)]
mod tests {
    use crate::{imx::hwid::Hwid, imx::signer::sign_wic};

    const TEST_HWID: Hwid = Hwid {
        cfg0: 0x6920_116d,
        cfg1: 0x4736_49d7,
        mac0: 0x3878_0e11,
        mac1: 0x66c5_0a99,
    };

    #[test]
    fn test() {
        use std::{
            fs::OpenOptions,
            io::{Read, Write},
        };

        let mut buf = Vec::new();

        let _ = OpenOptions::new().read(true).open("src/test.wic").unwrap().read_to_end(&mut buf).unwrap();
        let mut out = OpenOptions::new().create(true).truncate(true).write(true).open("test.wic.signed").unwrap();

        sign_wic(&mut buf, TEST_HWID).unwrap();
        out.write_all(&buf).unwrap();

        println!("Signed!")
    }
}

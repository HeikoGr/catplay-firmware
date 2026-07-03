use core::fmt;

use crate::mini::{
    crc::xor_crc_img,
    md5::{md5_calc, parse_md5_hex},
};

pub const HEADER_SIZE: usize = 131_072;
pub const BLOCK_SIZE: usize = 131_072;
pub const MAX_IMAGES: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OtaImageSection<'a> {
    pub ty: &'a str,
    pub name: &'a str,
    pub size: u32,
    pub md5: &'a str,
    pub crc: u32,
    pub offset: usize,
    pub data: &'a [u8],

    pub crc_fail: bool,
    pub md5_fail: bool,
}

impl<'a> fmt::Debug for OtaImageSection<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtaImageSection")
            .field("ty", &self.ty)
            .field("name", &self.name)
            .field("size", &self.size)
            .field("md5", &self.md5)
            .field("crc", &format_args!("{:#010x}", self.crc))
            .field("offset", &self.offset)
            .field("crc_fail", &self.crc_fail)
            .field("md5_fail", &self.md5_fail)
            .field("data", &format_args!("{} bytes", self.data.len()))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct OtaImageGlobals<'a> {
    pub bs_size: usize,
    pub packet_md5: Option<&'a str>,
    pub packet_md5_fail: bool,
}

impl<'a> OtaImageSection<'a> {
    pub fn data(&self, buf: &'a [u8]) -> &'a [u8] {
        &buf[self.offset..self.offset + self.size as usize]
    }
}

fn parse_ota<'a>(buf: &'a [u8], images: &mut [Option<OtaImageSection<'a>>], globals: &mut OtaImageGlobals<'a>) -> usize {
    let text = match str::from_utf8(&buf[..HEADER_SIZE.min(buf.len())]) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let mut count = 0;
    let mut offset = HEADER_SIZE;

    *globals = parse_globals(text);

    for part in text.split("\n\n") {
        if count >= images.len() {
            break;
        }
        if part.contains("img_name") {
            let mut name = "";
            let mut ty = "?";
            let mut size = 0u32;
            let mut md5 = "";
            let mut crc = 0u32;

            for line in part.lines() {
                let line = line.trim();
                if let Some(pos) = line.find('=') {
                    let (k, v) = line.split_at(pos);
                    let val = v[1..].trim();
                    match k.trim() {
                        "img_name" => name = val,
                        "img_type" => ty = val,
                        "img_size" => size = val.parse::<u32>().unwrap_or(0),
                        "img_md5" => md5 = val,
                        "img_crc" => crc = val.parse::<u32>().unwrap_or(0),
                        _ => {}
                    }
                }
            }

            if offset + size as usize > buf.len() {
                // debug!("OTA section: offset too big");
                return count;
            }

            let data = &buf[offset..offset + size as usize];
            let mut section = OtaImageSection {
                ty,
                name,
                size,
                md5,
                crc,
                offset,
                data,

                crc_fail: false,
                md5_fail: false,
            };

            if crc != 0 && xor_crc_img(data, globals.bs_size) != crc {
                section.crc_fail = true;
                // debug!("OTA Section: CRC fail");
            } else {
                // debug!("OTA Section: CRC OK");
            }

            if !md5.is_empty() {
                let md5_hex = parse_md5_hex(md5);
                if let Some(md5_hex) = md5_hex {
                    if md5_hex != md5_calc(data) {
                        // debug!("OTA Section: MD5 fail");
                        section.md5_fail = true;
                    } else {
                        // debug!("OTA Section: MD5 OK");
                    }
                } else {
                    section.md5_fail = true;
                    // debug!("OTA Section: MD5 fail (unparsable)");
                }
            }

            images[count].replace(section);

            count += 1;
            offset += size as usize;
        }
    }

    if let Some(packet_md5) = globals.packet_md5 {
        let parsed_global_md5 = parse_md5_hex(packet_md5);
        let global_md5 = md5_calc(&buf[HEADER_SIZE..offset]);

        if parsed_global_md5 != Some(global_md5) {
            // debug!("OTA Header: packet MD5 fail");
            globals.packet_md5_fail = true;
        }
    }

    count
}

fn parse_globals<'a>(text: &'a str) -> OtaImageGlobals<'a> {
    let mut bs_size = BLOCK_SIZE;
    let mut packet_md5 = None;
    for line in text.lines().map(|l| l.trim()) {
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim();
            match k.trim() {
                "bs_size" => bs_size = v.parse().unwrap_or(bs_size),
                "ota_img_packet_md5" => packet_md5 = Some(v),
                _ => {}
            }
        }
    }

    OtaImageGlobals {
        bs_size,
        packet_md5,
        packet_md5_fail: false,
    }
}

pub struct OtaExtractor<'a> {
    buf: &'a [u8],
    sections: [Option<OtaImageSection<'a>>; MAX_IMAGES],
    globals: OtaImageGlobals<'a>,
    parsed: bool,
}

impl<'a> OtaExtractor<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            sections: [None; MAX_IMAGES],
            globals: OtaImageGlobals::default(),
            parsed: false,
        }
    }

    pub fn parse(&mut self) -> Result<(), &'static str> {
        let n = parse_ota(self.buf, &mut self.sections, &mut self.globals);
        if n == 0 {
            return Err("file has 0 OTA sections");
        }

        for i in 0..n {
            let Some(image) = self.sections[i] else {
                continue;
            };

            if image.crc_fail {
                return Err("failed section crc check");
            }

            if image.md5_fail {
                return Err("failed section md5 check");
            }
        }

        if self.globals.packet_md5_fail {
            return Err("failed packet md5 check");
        }

        self.parsed = true;
        Ok(())
    }

    pub fn globals(&self) -> &OtaImageGlobals<'a> {
        &self.globals
    }

    pub fn sections(&self) -> &[Option<OtaImageSection<'a>>; MAX_IMAGES] {
        &self.sections
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self};

    use super::*;
    extern crate std;

    #[test]
    fn test_parse() {
        let file = "/tmp/ota.bin";

        let data = fs::read(file).unwrap();

        let mut parser = OtaExtractor::new(&data);
        parser.parse().unwrap();

        parser.sections[0].as_mut().unwrap().data = &[];
        parser.sections[1].as_mut().unwrap().data = &[];

        assert_eq!(
            parser.sections().iter().flatten().collect::<Vec<_>>(),
            [
                Some(OtaImageSection {
                    ty: "kernel",
                    name: "xImage",
                    size: 3088448,
                    md5: "7e5f7b665e18ddfe948dbfba69f6a398",
                    crc: 99439955,
                    offset: 131072,
                    crc_fail: false,
                    md5_fail: false,

                    data: &[]
                }),
                Some(OtaImageSection {
                    ty: "rootfs",
                    name: "rootfs.squashfs",
                    size: 12054528,
                    md5: "efda6cd62dd10202545e36f07d4a85b3",
                    crc: 3804379493,
                    offset: 3219520,
                    crc_fail: false,
                    md5_fail: false,

                    data: &[]
                }),
                None
            ]
            .iter()
            .flatten()
            .collect::<Vec<_>>()
        );

        assert_eq!(
            parser.globals(),
            &OtaImageGlobals {
                bs_size: 131072,
                packet_md5: Some("fe5800799de0acba6d778ffd895b4e81"),
                packet_md5_fail: false
            }
        );
    }
}

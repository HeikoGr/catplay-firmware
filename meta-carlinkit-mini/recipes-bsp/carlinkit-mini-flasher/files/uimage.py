import struct
import zlib

from errors import X1600UsbBootError

UIMAGE_MAGIC = 0x27051956
UIMAGE_HEADER_SIZE = 64

class UImage:
    """
    Classic legacy uImage header, 64B, big-endian.

    Layout:
        uint32_t ih_magic;
        uint32_t ih_hcrc;
        uint32_t ih_time;
        uint32_t ih_size;
        uint32_t ih_load;
        uint32_t ih_ep;
        uint32_t ih_dcrc;
        uint8_t  ih_os;
        uint8_t  ih_arch;
        uint8_t  ih_type;
        uint8_t  ih_comp;
        uint8_t  ih_name[32];
    """

    def __init__(self, data: bytes, verify_header_crc: bool = True, verify_data_crc: bool = False):
        if len(data) < UIMAGE_HEADER_SIZE:
            raise X1600UsbBootError("kernel.bin is too small, this does not look like a uImage")

        header = data[:UIMAGE_HEADER_SIZE]

        (
            magic,
            header_crc,
            timestamp,
            size,
            load_addr,
            entry_addr,
            data_crc,
            os_type,
            arch,
            image_type,
            comp,
            name_raw,
        ) = struct.unpack(">7I4B32s", header)

        if magic != UIMAGE_MAGIC:
            raise X1600UsbBootError(f"Invalid uImage magic: 0x{magic:08x}")

        payload = data[UIMAGE_HEADER_SIZE : UIMAGE_HEADER_SIZE + size]
        if len(payload) != size:
            raise X1600UsbBootError(
                f"uImage payload truncated: header says {size} B, file contains {len(payload)} B"
            )

        if verify_header_crc:
            header_for_crc = bytearray(header)
            header_for_crc[4:8] = b"\x00\x00\x00\x00"
            calc_hcrc = zlib.crc32(header_for_crc) & 0xFFFFFFFF
            if calc_hcrc != header_crc:
                raise X1600UsbBootError(
                    f"Invalid uImage header CRC: expected=0x{header_crc:08x}, "
                    f"calc=0x{calc_hcrc:08x}"
                )

        if verify_data_crc:
            calc_dcrc = zlib.crc32(payload) & 0xFFFFFFFF
            if calc_dcrc != data_crc:
                raise X1600UsbBootError(
                    f"Invalid uImage data CRC: expected=0x{data_crc:08x}, "
                    f"calc=0x{calc_dcrc:08x}"
                )

        self.magic = magic
        self.header_crc = header_crc
        self.timestamp = timestamp
        self.size = size
        self.load_addr = load_addr
        self.entry_addr = entry_addr
        self.data_crc = data_crc
        self.os_type = os_type
        self.arch = arch
        self.image_type = image_type
        self.comp = comp
        self.name = name_raw.rstrip(b"\x00").decode("ascii", errors="replace")
        self.data = payload

    def __str__(self) -> str:
        return (
            f"uImage(name='{self.name}', size={self.size}, "
            f"load=0x{self.load_addr:08x}, entry=0x{self.entry_addr:08x}, "
            f"os={self.os_type}, arch={self.arch}, type={self.image_type}, comp={self.comp})"
        )

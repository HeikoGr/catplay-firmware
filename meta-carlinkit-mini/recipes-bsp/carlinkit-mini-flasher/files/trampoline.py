import struct
from dataclasses import dataclass

@dataclass
class MipsFwArgsLayout:
    blob: bytes
    argc: int
    argv_addr: int
    envp_addr: int
    promvec: int
    cmdline: str

    @staticmethod
    def _align_up(value: int, align: int) -> int:
        return (value + align - 1) & ~(align - 1)

    @staticmethod
    def build(base_addr: int, cmdline: str) -> "MipsFwArgsLayout":
        """
        Builds classic firmware args block for Linux/MIPS:
          a0 = argc
          a1 = argv (char **)
          a2 = envp (char **)
          a3 = promvec (0)

        argv[0] = "linux"
        argv[1] = cmdline
        argv[2] = NULL
        envp[0] = NULL
        """
        argv_items = [b"linux\x00", cmdline.encode("utf-8") + b"\x00"]
        argc = len(argv_items)

        argv_table_size = (argc + 1) * 4
        envp_table_size = 4  # only NULL
        strings_offset = MipsFwArgsLayout._align_up(argv_table_size + envp_table_size, 4)

        total_strings = sum(len(item) for item in argv_items)
        blob = bytearray(strings_offset + total_strings)

        write_off = strings_offset
        argv_ptrs: list[int] = []
        for item in argv_items:
            argv_ptrs.append(base_addr + write_off)
            blob[write_off : write_off + len(item)] = item
            write_off += len(item)

        for idx, ptr in enumerate(argv_ptrs):
            struct.pack_into("<I", blob, idx * 4, ptr)
        struct.pack_into("<I", blob, argc * 4, 0)  # argv terminator
        struct.pack_into("<I", blob, argv_table_size, 0)  # envp[0] = NULL

        return MipsFwArgsLayout(
            blob=bytes(blob),
            argc=argc,
            argv_addr=base_addr,
            envp_addr=base_addr + argv_table_size,
            promvec=0,
            cmdline=cmdline,
        )


class Trampoline:
    @staticmethod
    def _mips_i(op: int, rs: int, rt: int, imm16: int) -> int:
        return ((op & 0x3F) << 26) | ((rs & 0x1F) << 21) | ((rt & 0x1F) << 16) | (imm16 & 0xFFFF)

    @staticmethod
    def _mips_r(rs: int, rt: int, rd: int, shamt: int, funct: int) -> int:
        return ((rs & 0x1F) << 21) | ((rt & 0x1F) << 16) | ((rd & 0x1F) << 11) | ((shamt & 0x1F) << 6) | (funct & 0x3F)

    @classmethod
    def _encode_lui(cls, rt: int, imm16: int) -> int:
        return cls._mips_i(0x0F, 0, rt, imm16)

    @classmethod
    def _encode_ori(cls, rt: int, rs: int, imm16: int) -> int:
        return cls._mips_i(0x0D, rs, rt, imm16)

    @classmethod
    def _encode_jr(cls, rs: int) -> int:
        return cls._mips_r(rs, 0, 0, 0, 0x08)

    @classmethod
    def _mips_load_u32(cls, reg: int, value: int) -> list[int]:
        hi = (value >> 16) & 0xFFFF
        lo = value & 0xFFFF
        return [
            cls._encode_lui(reg, hi),
            cls._encode_ori(reg, reg, lo),
        ]

    @classmethod
    def build_linux_mips(
        cls,
        kernel_entry: int,
        a0_val: int,
        a1_val: int,
        a2_val: int,
        a3_val: int,
    ) -> bytes:
        """
        MIPS32 little-endian trampoline:
          - loads a0..a3,
          - loads kernel entry address to t9,
          - executes jr t9; nop.
        """
        regs = {
            "a0": 4,
            "a1": 5,
            "a2": 6,
            "a3": 7,
            "t9": 25,
        }

        words: list[int] = []
        words.extend(cls._mips_load_u32(regs["a0"], a0_val))
        words.extend(cls._mips_load_u32(regs["a1"], a1_val))
        words.extend(cls._mips_load_u32(regs["a2"], a2_val))
        words.extend(cls._mips_load_u32(regs["a3"], a3_val))
        words.extend(cls._mips_load_u32(regs["t9"], kernel_entry))
        words.append(cls._encode_jr(regs["t9"]))
        words.append(0x00000000)  # branch delay slot: nop
        return b"".join(struct.pack("<I", w) for w in words)

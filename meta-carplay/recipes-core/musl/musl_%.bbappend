# Backport musl 1.2.6 for riscv32 support.
BASEVER = "1.2.6"
SRCREV = "9fa28ece75d8a2191de7c5bb53bed224c5947417"
LIC_FILES_CHKSUM = "file://COPYRIGHT;md5=0c2904cdc34777fb4067732bae145506"

FILESEXTRAPATHS:prepend := "${THISDIR}/${BPN}:"

# Wrynose fetches directly from the canonical musl host. The old Scarthgap
# source override must not be retained because it adds the same SCM twice.

# Keep using the local 0001/0002 patch copies from FILESEXTRAPATHS; both apply
# cleanly on 1.2.6. 0003 is included upstream.
SRC_URI:remove = "file://0003-elf.h-add-typedefs-for-Elf64_Relr-and-Elf32_Relr.patch"

# musl 1.2.6 supports riscv32; drop the 1.2.4 compatibility block.
COMPATIBLE_HOST:riscv32 = ""

# Latest version of clang miscompiles musl on riscv32, corrupting code which performs syscalls
TOOLCHAIN:riscv32 = "gcc"

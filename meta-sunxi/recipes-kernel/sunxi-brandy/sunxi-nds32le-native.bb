SUMMARY = "Prebuilt Andes toolchain for Allwinner Brandy"
LICENSE = "CLOSED"

inherit native

COMPATIBLE_HOST = "x86_64.*-linux"
PV = "5.3.0"

SRC_URI = "https://github.com/andestech/Andes-Development-Kit/releases/download/ast-v5_3_0-release-linux/nds32le-linux-glibc-v5d.txz;unpack=0"
SRC_URI[sha256sum] = "5eecef9684ccc75302809ccfa8b1419300d6ea26d697434c870672d70210ff59"
S = "${UNPACKDIR}"

DEPENDS = "xz-native"

# Preserve the vendor layout, including target libraries and relative symlinks.
INHIBIT_SYSROOT_STRIP = "1"
TOOLCHAIN_INSTALL_DIR = "${libdir}/sunxi-toolchains"

do_configure[noexec] = "1"
do_compile[noexec] = "1"

do_install() {
    install -d "${D}${TOOLCHAIN_INSTALL_DIR}"
    tar --no-same-owner -xJf "${DL_DIR}/nds32le-linux-glibc-v5d.txz" \
        -C "${D}${TOOLCHAIN_INSTALL_DIR}"
    test -x "${D}${TOOLCHAIN_INSTALL_DIR}/nds32le-linux-glibc-v5d/bin/riscv32-unknown-linux-gcc"
    test -f "${D}${TOOLCHAIN_INSTALL_DIR}/nds32le-linux-glibc-v5d/lib/gcc/riscv32-linux/12.2.0/include/stddef.h"
}

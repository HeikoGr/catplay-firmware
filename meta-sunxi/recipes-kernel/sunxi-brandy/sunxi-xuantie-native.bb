SUMMARY = "Prebuilt Xuantie toolchain for Allwinner FES"
LICENSE = "CLOSED"

inherit native

COMPATIBLE_HOST = "x86_64.*-linux"
PV = "2.8.1"

# The vendor repository carries this toolchain as a nested archive.
SRC_URI = "git://github.com/sam-yangjj/tina-v821-v1.3-brandy.git;branch=main;protocol=https"
SRCREV = "34809037526678ccda1720cb4b4ec7ee32272c38"
S = "${UNPACKDIR}/${BP}/brandy-2.0/tools/toolchain"

DEPENDS = "xz-native"

# Preserve the vendor layout, including target libraries and relative symlinks.
INHIBIT_SYSROOT_STRIP = "1"
TOOLCHAIN_INSTALL_DIR = "${libdir}/sunxi-toolchains"
XUANTIE_TOOLCHAIN_NAME = "Xuantie-900-gcc-linux-5.10.4-glibc-x86_64-V2.8.1"

do_configure[noexec] = "1"
do_compile[noexec] = "1"

do_install() {
    install -d "${D}${TOOLCHAIN_INSTALL_DIR}"
    tar --no-same-owner -xJf "${S}/${XUANTIE_TOOLCHAIN_NAME}.tar.xz" \
        -C "${D}${TOOLCHAIN_INSTALL_DIR}"
    test -x "${D}${TOOLCHAIN_INSTALL_DIR}/${XUANTIE_TOOLCHAIN_NAME}/bin/riscv64-unknown-linux-gnu-gcc"
    test -f "${D}${TOOLCHAIN_INSTALL_DIR}/${XUANTIE_TOOLCHAIN_NAME}/lib/gcc/riscv64-unknown-linux-gnu/10.4.0/include/stddef.h"
}

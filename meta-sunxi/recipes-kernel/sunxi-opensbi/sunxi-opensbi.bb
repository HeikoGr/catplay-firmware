SUMMARY = "Allwinner OpenSBI binary for sun300iw1p1/V821"
DESCRIPTION = "Exports the vendor OpenSBI binary from the Tina V821 device tree."
LICENSE = "CLOSED"

inherit deploy

PROVIDES += "virtual/opensbi"
DEPENDS += "gzip-native"
PR = "r3"
SRC_URI = "git://github.com/sam-yangjj/tina-v821-v1.3-device.git;branch=main;protocol=https"

SRCREV = "e77a4bd91bc3ba43fd294a00c75ee8bd9d7a1b94"
PV = "1.3git${SRCPV}"

OPENSBI_BIN = "config/chips/v821/bin/opensbi_sun300iw1p1.bin"
OPENSBI_INSTALL_NAME = "opensbi-v821.bin"

PACKAGE_ARCH = "${MACHINE_ARCH}"

do_configure[noexec] = "1"
do_compile[noexec] = "1"

do_install() {
    install -d "${D}${nonarch_base_libdir}/firmware/sunxi-opensbi"
    install -m 0644 "${S}/${OPENSBI_BIN}" \
        "${D}${nonarch_base_libdir}/firmware/sunxi-opensbi/${OPENSBI_INSTALL_NAME}"
    # gzip -9 -c "${S}/${OPENSBI_BIN}" \
    #   > "${D}${nonarch_base_libdir}/firmware/sunxi-opensbi/${OPENSBI_INSTALL_NAME}.gz"
}

FILES:${PN} += "${nonarch_base_libdir}/firmware/sunxi-opensbi"

PACKAGE_ARCH = "${MACHINE_ARCH}"

COMPATIBLE_MACHINE = "(woo|woo-recov)"

do_deploy() {
    install -d "${DEPLOYDIR}/sunxi-opensbi"
    install -m 0644 "${S}/${OPENSBI_BIN}" \
        "${DEPLOYDIR}/sunxi-opensbi/${OPENSBI_INSTALL_NAME}"
    gzip -9 -c "${S}/${OPENSBI_BIN}" \
        > "${DEPLOYDIR}/sunxi-opensbi/${OPENSBI_INSTALL_NAME}.gz"
}

addtask do_deploy before do_build after do_install

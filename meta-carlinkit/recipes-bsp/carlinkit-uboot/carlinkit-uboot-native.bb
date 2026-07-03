DESCRIPTION = "Carlinkit C2A bootloader - original signed IMX files for secure boot"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/files/common-licenses/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

SRC_URI += "file://u-boot-signed.csf"
SRC_URI += "file://u-boot-signed.imx"
SRC_URI += "file://u-boot-signed.imx.nohdr"

S = "${WORKDIR}"
PV = "1.0"

inherit native

do_install() {
    install -Dm 0755 ${S}/u-boot-signed.imx ${D}${libexecdir}/u-boot-carlinkit.imx
    install -Dm 0755 ${S}/u-boot-signed.imx.nohdr ${D}${libexecdir}/u-boot-carlinkit.imx.nohdr
    install -Dm 0755 ${S}/u-boot-signed.csf ${D}${libexecdir}/u-boot-carlinkit.csf
}
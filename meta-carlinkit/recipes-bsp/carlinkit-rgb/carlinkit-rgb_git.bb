SUMMARY = "Kernel driver for WS2812B RGB LED"
DESCRIPTION = "Kernel driver for WS2812B RGB LED"
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

inherit module

MODULE_NAME = "carlinkit_rgb"
EXTRA_OEMAKE += "-C ${STAGING_KERNEL_DIR} M=${S} KCFLAGS=-O2"

FILESEXTRAPATHS:prepend := "${THISDIR}/src:"
SRC_URI = "file://carlinkit_rgb.c file://Makefile"

S = "${WORKDIR}"
B = "${WORKDIR}/build"

# do_install() {
#     install -Dm 0644 ${S}/${MODULE_NAME}.ko ${D}${nonarch_base_libdir}/modules/${KERNEL_VERSION}/kernel/drivers/extra/${MODULE_NAME}.ko
# }

#FILES:${PN} = "/lib/modules/${KERNEL_VERSION}/kernel/drivers/extra/${MODULE_NAME}.ko"

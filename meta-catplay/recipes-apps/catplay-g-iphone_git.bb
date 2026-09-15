SUMMARY = "Kernel driver for CatPlay"
DESCRIPTION = "Kernel driver for CatPlay"
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

inherit catplay-src-bundle-local

inherit module

MODULE_NAME = "g_iphone"
EXTRA_OEMAKE += "-C ${STAGING_KERNEL_DIR} M=${S} KCFLAGS=-O2"

FILESEXTRAPATHS:prepend := "${THISDIR}/src:"

# Applied inside ${S}. Meant for catplay-labs/catplay; carried here until it
# lands there.
SRC_URI:append = " \
    file://0001-g_iphone-role-switch-only-after-the-host-took-our-ACK-and-recover-after-a-failed-switch.patch \
"

S = "${UNPACKDIR}/usb/catplay_iap2_usb_host/g_iphone"

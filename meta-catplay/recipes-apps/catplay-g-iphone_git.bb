SUMMARY = "Kernel driver for CatPlay"
DESCRIPTION = "Kernel driver for CatPlay"
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

inherit catplay-src-bundle-local

inherit module

MODULE_NAME = "g_iphone"
EXTRA_OEMAKE += "-C ${STAGING_KERNEL_DIR} M=${S} KCFLAGS=-O2"

FILESEXTRAPATHS:prepend := "${THISDIR}/src:"

# Patches apply inside ${S}, i.e. usb/catplay_iap2_usb_host/g_iphone of the
# CatPlay source bundle.
SRC_URI:append = " \
    file://0001-g_iphone-answer-class-requests-our-descriptors-promise.patch \
    file://0002-g_iphone-switch-roles-only-after-the-host-took-our-ACK.patch \
"

S = "${UNPACKDIR}/usb/catplay_iap2_usb_host/g_iphone"

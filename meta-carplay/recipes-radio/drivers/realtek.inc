REALTEK_MODULE ?= "88x2cs"
REALTEK_CFLAGS ?= ""
REALTEK_TARGET ?= "RTL8822CS"

SUMMARY ?= "Driver for ${REALTEK_TARGET}"
DESCRIPTION ?= "Realtek vendor driver for ${REALTEK_TARGET}"
LICENSE ?= "GPL-2.0-only"
LIC_FILES_CHKSUM ?= "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

MAKE_TARGETS = "modules"
MODULES_INSTALL_TARGET = "modules"

EXTRA_OEMAKE += "-C ${STAGING_KERNEL_DIR} M=${S}"
EXTRA_OEMAKE += "${@' '.join(['USER_EXTRA_CFLAGS+={}'.format(flag) for flag in d.getVar('REALTEK_CFLAGS').split()])}"
EXTRA_OEMAKE += "CONFIG_${REALTEK_TARGET}=m"

inherit module
require module-extra.inc

# SRC_URI:remove = "*"
# SRC_URI:append = " https://github.com/Freescale/linux-fslc/archive/7a7717289cd3915e484b02c02557de380352b030.tar.gz"
# SRC_URI[sha256sum] = "a7ff167511318381ab82908c0ccb753cf80a5148ec2abf0cb4d9803e0067389b"
# S = "${WORKDIR}/linux-fslc-7a7717289cd3915e484b02c02557de380352b030"
#DEBUG_BUILD:forcevariable = "1"

FILESEXTRAPATHS:append := "${THISDIR}/files:"

#LINUX_VERSION = "6.12.34"
#KBRANCH = "6.12.x+fslc"
#SRCREV = "e92f5b7050c74e8052f071fd2f1d233d9a4b2f30"

SRC_URI = " git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git;branch=${KBRANCH};protocol=https"
KBRANCH = "linux-6.12.y"
SRCREV = "b0c51e95f54e5f4e13a7ada6629125b0bc427a96"


SRC_URI:append = " file://0001-Unlock-experimental-LTO-for-arm32.patch"
SRC_URI:append = " file://0002-Change-Os-to-more-aggressive-Oz-for-CONFIG_CC_OPTIMIZE_FOR_SIZE.patch"
SRC_URI:append = " file://0003-Fix-early-UART-for-IMX6.patch"
SRC_URI:append = " file://0004-Ignore-VBUS-detection-requirement-in-gadget-mode-on-.patch"
SRC_URI:append = " file://0005-cdc-ncm-fix-linking-with-generic-CarPlay-Headunits.patch"
SRC_URI:append = " file://0006-erofs-support-for-initrd.patch"
SRC_URI:append = " file://0007-phy-mxs-usb-disable-charger-detection.patch"
SRC_URI:append = " file://0008-Whitelist-mx25l12805d-and-mx25u12835f-for-QSPI-support.patch"
SRC_URI:append = " file://0009-btbcm-recognize-BCM4358A3.patch"
SRC_URI:append = " file://0010-chipidea-fix-role-switch-delay.patch"
SRC_URI:append = " file://0011-crypto-remove-forced-entropy-collection-that-adds-0.5s-lag-to-system-boot.patch"

do_kernel_metadata:prepend() {
    bbwarn "Using defconfig: ${KBUILD_DEFCONFIG}"
}

# Fix virtual/kernel
COMPATIBLE_MACHINE:append = "|qemuarm"

inherit kernel-clang-c2a
inherit kernel-deploy-extras-c2a
inherit kernel-deploy-modules-to-sysroot-c2a
inherit kernel-firmware-stage-c2a
inherit kernel-firmware-conflicts-c2a
inherit kernel-extra-config-c2a
inherit kernel-broken-version-c2a

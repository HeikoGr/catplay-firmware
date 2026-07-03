inherit image
inherit image-cleanup
inherit image-upxify
inherit image-erofs
inherit image-bootmark

SUMMARY = "Minimal C2A CarPlay system"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/files/common-licenses/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

PACKAGE_EXCLUDE:append = " kernel-image-*"

# Reduce garbage
IMAGE_LINGUAS = ""
IMAGE_FEATURES = "read-only-rootfs debug-tweaks empty-root-password allow-empty-password allow-root-login"

MACHINE_FIRMWARE ??= "" 

PACKAGE_INSTALL:append = " \
    packagegroup-core-c2a \
    kernel-modules \
    ${MACHINE_FIRMWARE} \
"

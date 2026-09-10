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
IMAGE_FEATURES = "read-only-rootfs empty-root-password allow-empty-password allow-root-login post-install-logging"

MACHINE_FIRMWARE ??= "" 

# Mountpoints that must exist in the immutable core filesystem.  Runtime
# helpers such as tmpfiles cannot create these on a read-only EROFS root.
C2A_CORE_MOUNTPOINTS ?= "/persist"

ROOTFS_POSTPROCESS_COMMAND += "create_c2a_core_mountpoints; "

create_c2a_core_mountpoints() {
	for mountpoint in ${C2A_CORE_MOUNTPOINTS}; do
		install -d -m 0755 "${IMAGE_ROOTFS}${mountpoint}"
	done
}

PACKAGE_INSTALL:append = " \
    packagegroup-core-c2a \
    kernel-modules \
    ${MACHINE_FIRMWARE} \
"

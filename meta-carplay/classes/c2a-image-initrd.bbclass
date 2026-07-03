# don't default to "rootfs"
IMAGE_NAME_SUFFIX = ""

ZSTD_COMPRESSION_LEVEL = "-10"
# Add --no-check to avoid duplicating CRC work that's already done at fitImage level
ZSTD_DEFAULTS:append = " --no-check"

python () {
    types = filter(None, [
        d.getVar("C2A_INITRAMFS_FSTYPES"),
        d.getVar("C2A_EXTRA_FSTYPES"),
    ])
    d.setVar("IMAGE_FSTYPES", " ".join(types))
}

inherit c2a-x-ro

# Reduce glitching between kernel release and kernel modules on rootfs...
KERNELRELEASE_STAMP = "${@oe.utils.read_file(d.expand('${STAGING_KERNEL_BUILDDIR}/include/config/kernel.release')).strip() if os.path.exists(d.expand('${STAGING_KERNEL_BUILDDIR}/include/config/kernel.release')) else ''}"
do_rootfs[depends] += "virtual/kernel:do_packagedata"
do_rootfs[depends] += "virtual/kernel:do_package_write_ipk"
do_rootfs[vardeps] += "KERNELRELEASE_STAMP"

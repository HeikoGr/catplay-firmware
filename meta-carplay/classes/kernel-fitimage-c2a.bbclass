# Patch kernel-fitimage.bbclass
FIT_SUPPORTED_INITRAMFS_FSTYPES:append = " erofs-lz4hc erofs-lz4"

C2A_INITRAMFS_FSTYPES ??= "erofs-lz4hc"

python () {
    d.setVar("INITRAMFS_FSTYPES", d.getVar("C2A_INITRAMFS_FSTYPES"))
    d.setVar("INITRAMFS_IMAGE_BUNDLE", "0")
    d.setVar("INITRAMFS_IMAGE", d.getVar("C2A_INITRAMFS_IMAGE"))
}

inherit kernel-fitimage

# -19 is too slow to decompress
C2A_KERNEL_ZSTD_LEVEL ??= "-5"
# sha256 is too slow to check
FIT_HASH_ALG:forcevariable = "crc32"

uboot_prep_kimage() {
    # upstream forcefully chooses zImage to be put inside fitImage
    # which then ignores dtb that's coming from the same fitImage because it zeroes out r2 register
    # also, it doesn't allow usage of zstd
    # so we fix both of these issues here

    bbwarn "✂️ Patching uboot_prep_kimage to fix fitImage!"

    IMG_SRC="${B}/arch/arm/boot/Image"

	output_dir=$1
	# For backward compatibility with kernel-fitimage.bbclass and kernel-uboot.bbclass
	# support calling without parameter as well
	if [ -z "$output_dir" ]; then
		output_dir='.'
	fi
	linux_bin=$output_dir/linux.bin

    rm -f "${linux_bin}"
    # Add --no-check to avoid duplicating CRC work that's already done at fitImage level
    zstd --no-check --threads=${ZSTD_THREADS} "${C2A_KERNEL_ZSTD_LEVEL}" -o "${linux_bin}" "${IMG_SRC}"
    linux_comp="zstd"

	printf "$linux_comp" > "$output_dir/linux_comp"
}

C2A_PATCHED_PREP_KIMAGE = "1"
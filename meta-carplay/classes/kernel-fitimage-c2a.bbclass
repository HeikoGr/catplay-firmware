# Kernel-side producer for the standalone linux-yocto-fitimage recipe.
#
# Since Yocto 6.0, FIT assembly no longer runs inside the kernel recipe.
# This wrapper keeps the C2A-specific kernel preparation while the standalone
# recipe consumes the deployed linux.bin and linux_comp artifacts.
inherit kernel-fit-extra-artifacts

# uboot_prep_kimage uses zstd directly during virtual/kernel:do_deploy.
do_deploy[depends] += "zstd-native:do_populate_sysroot"

# -19 is too slow to decompress
C2A_KERNEL_ZSTD_LEVEL ??= "-5"
uboot_prep_kimage() {
    # Keep using the raw ARM Image. A zImage clears r2 before jumping to the
    # kernel, which would discard the DTB selected by the surrounding FIT.
    # Compress the raw image with zstd, as expected by the C2A U-Boot flow.

    bbnote "Preparing the C2A zstd-compressed kernel payload for fitImage"

    IMG_SRC="${B}/arch/arm/boot/Image"

	output_dir=$1
	# Keep compatibility with kernel-uboot callers which omit the argument.
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

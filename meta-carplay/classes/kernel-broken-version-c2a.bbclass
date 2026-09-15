# Version suffix generator seems broken beyond repair (generates double -fslc-fslc)
# but generates -fslc for out-of-tree modules, causing vermagic mismatch
do_kernel_localversion[noexec] = "1"
LOCALVERSION = ""
KERNEL_VERSION_SANITY_SKIP = "1"

# Reduce sstate glitching where packages generate different kernel hash for the same package version
#
# CONFIG_LOCALVERSION is a Kconfig *string* option, so its .config value has to
# be quoted. Without the quotes Kconfig discards the line and the kernel ends up
# with an empty localversion - which is what used to happen here, and why dmesg
# showed a bare "Linux version 6.18.36" with no suffix at all.
#
# Keep this value stable. It feeds UTS_RELEASE and therefore module vermagic;
# putting anything build-specific in here would break out-of-tree modules
# (aic8800, iap2_char) on every commit. Build identity goes into
# KBUILD_BUILD_TIMESTAMP instead - see linux-letux_6.18.bb.
C2A_KERNEL_EXTRA_CONFIG[CONFIG_LOCALVERSION] = '"-c2a"'
C2A_KERNEL_EXTRA_CONFIG[CONFIG_LOCALVERSION_AUTO] = "n"

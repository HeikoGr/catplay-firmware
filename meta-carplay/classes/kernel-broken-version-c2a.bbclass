# Version suffix generator seems broken beyond repair (generates double -fslc-fslc)
# but generates -fslc for out-of-tree modules, causing vermagic mismatch
do_kernel_localversion[noexec] = "1"
LOCALVERSION = ""
KERNEL_VERSION_SANITY_SKIP = "1"

# Reduce sstate glitching where packages generate different kernel hash for the same package version
C2A_KERNEL_EXTRA_CONFIG[CONFIG_LOCALVERSION] = "-c2a"
C2A_KERNEL_EXTRA_CONFIG[CONFIG_LOCALVERSION_AUTO] = "n"

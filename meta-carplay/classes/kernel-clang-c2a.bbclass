inherit ccache

C2A_KERNEL_CLANG ?= "1"
C2A_KERNEL_CLANG_LLD ?= "${C2A_KERNEL_CLANG}"

# Broken relocations
C2A_KERNEL_CLANG_LLD:mipsarch = "0"

python () {
    clang = (d.getVar("C2A_KERNEL_CLANG") or "").strip()
    lld = (d.getVar("C2A_KERNEL_CLANG_LLD") or "").strip()

    if clang not in ("0", "1"):
        bb.fatal("C2A_KERNEL_CLANG must be '0' or '1'")
    if lld not in ("0", "1"):
        bb.fatal("C2A_KERNEL_CLANG_LLD must be '0' or '1'")

    if clang == "1":
        d.setVar("TOOLCHAIN:forcevariable", "clang")
        d.setVar("BUILD_OPTIMIZATION:forcevariable", "")
    else:
        d.setVar("TOOLCHAIN:forcevariable", "gcc")

    if lld == "1":
        d.setVar("KERNEL_LD:toolchain-clang", "${CCACHE}${TARGET_PREFIX}ld.lld")
    else:
        d.setVar("KERNEL_LD:toolchain-clang", "${CCACHE}${TARGET_PREFIX}ld.bfd")
}

DEPENDS:append:toolchain-clang = " clang-cross-${TARGET_ARCH}"
KERNEL_CC:toolchain-clang = "${CCACHE}${TARGET_PREFIX}clang ${HOST_CC_KERNEL_ARCH} ${DEBUG_PREFIX_MAP} -fno-integrated-as -fdebug-prefix-map=${STAGING_KERNEL_DIR}=${KERNEL_SRC_PATH}"

KERNEL_AR:toolchain-clang = "${CCACHE}${TARGET_PREFIX}llvm-ar"
KERNEL_NM:toolchain-clang = "${CCACHE}${TARGET_PREFIX}llvm-nm"
KERNEL_AS:toolchain-clang = "${CCACHE}${TARGET_PREFIX}llvm-as"
KERNEL_OBJCOPY:toolchain-clang = "${CCACHE}${TARGET_PREFIX}llvm-objcopy"
KERNEL_OBJDUMP:toolchain-clang = "${CCACHE}${TARGET_PREFIX}llvm-objdump"

# kernel class ignores NM/AR/AS
EXTRA_OEMAKE:append:toolchain-clang = ' NM="${KERNEL_NM}" AR="${KERNEL_AR}" AS="${KERNEL_AS}"'

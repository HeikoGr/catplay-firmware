# Backport of the pending OE-Core change:
# https://patchwork.yoctoproject.org/project/oe-core/patch/20260223064905.2417217-1-mark.yang@lge.com/
# GNU ld needs LLVMgold.so to consume LLVM LTO objects. LLVM only builds the
# plugin when the binutils plugin headers are available.
DEPENDS:append = " binutils"

EXTRA_OECMAKE:append = " -DLLVM_BINUTILS_INCDIR=${STAGING_INCDIR}"

PACKAGES =+ "llvm-linker-tools"
FILES:llvm-linker-tools = "${libdir}/LLVMgold* ${libdir}/libLTO.so.*"

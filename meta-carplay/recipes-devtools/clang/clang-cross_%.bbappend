# The clang driver derives the GNU LTO plugin path from its own location.
# OE installs cross drivers one level deeper than regular native tools:
#
#   ${prefix_native}/bin/${TARGET_SYS}/${TARGET_PREFIX}clang
#
# Consequently clang walks from ${bindir} to ${bindir}/../lib/LLVMgold.so,
# while llvm-native correctly stages the plugin in ${libdir}.  Keep a relative
# link at the path expected by the driver; the plugin itself remains owned by
# LLVM.  Do not use ${bindir}/lib here: cross.bbclass has already appended the
# target tuple to ${bindir}.
SYSROOT_DIRS:append = " ${exec_prefix}/bin/lib"

do_install:append() {
    install -d ${D}${exec_prefix}/bin/lib
    ln -snf ../../lib/LLVMgold.so ${D}${exec_prefix}/bin/lib/LLVMgold.so
}

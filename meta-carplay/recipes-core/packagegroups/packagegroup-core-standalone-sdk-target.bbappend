# The clang SDK packagegroup pulls compiler-rt development packages even
# when the target uses the GNU runtime. compiler-rt is intentionally disabled
# for ARMv5, so do not make the standalone SDK unbuildable on this machine.
RRECOMMENDS:${PN}:remove:toolchain-clang:armv5 = " libcxx-dev libcxx-staticdev compiler-rt-dev compiler-rt-staticdev"

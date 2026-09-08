# ARMv5 has no native LLVM atomic lowering for the operations used by SDL2.
# Clang emits the compiler-rt/libatomic entry points, so link against the GNU
# libatomic implementation when using the GNU runtime on this machine.
DEPENDS:append:armv5 = " gcc-runtime"
LDFLAGS:append:armv5 = " -Wl,--no-as-needed -latomic -Wl,--as-needed"

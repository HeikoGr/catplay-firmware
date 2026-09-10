# With TC_CXX_RUNTIME="llvm", compiler-rt builds llvm-libgcc and therefore
# owns libunwind.a (libgcc_eh.a links to it).  OE-Core's libcxx recipe also
# installs its locally built static unwind archive on musl, which makes both
# recipes export the same sysroot path.  Keep the local build available while
# linking libcxx, but leave installation and sysroot ownership to compiler-rt.
do_install:append:class-target:libc-musl() {
    if ${@bb.utils.contains('TC_CXX_RUNTIME', 'llvm', 'true', 'false', d)}; then
        rm -f ${D}${libdir}/libunwind.a
    fi
}

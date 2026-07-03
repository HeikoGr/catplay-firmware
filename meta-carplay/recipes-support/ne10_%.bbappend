EXTRA_OECMAKE:remove = '-DNE10_BUILD_SHARED=ON'

do_install() {
    install -d ${D}${libdir}
    install -d ${D}${includedir}
    install -m 0644 ${S}/inc/NE10*.h ${D}${includedir}/
    install -m 0644 ${B}/modules/libNE10.a ${D}${libdir}/
   # install -m 0755 ${B}/modules/libNE10.so.* ${D}${libdir}/
    #cp -d ${B}/modules/libNE10.so ${D}${libdir}/
}

COMPATIBLE_MACHINE:mipsarch = "(.*)"
NE10_TARGET_ARCH:mipsarch = ""
EXTRA_OECMAKE:mipsarch = " \
    -DGNULINUX_PLATFORM=ON \
    -DNE10_ENABLE_NEON=OFF \
    -DNE10_ENABLE_MSA=OFF \
    -DNE10_BUILD_TESTS=OFF \
"

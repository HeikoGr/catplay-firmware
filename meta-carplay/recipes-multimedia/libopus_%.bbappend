EXTRA_OECONF:append:armv7a = " --disable-shared --enable-static --disable-examples"
EXTRA_OECONF:remove:armv7a = "--enable-shared"
DEPENDS:armv7a += " ne10"

do_install:append:armv7a() {
    sed -i 's/^Libs:.*/& -lNE10/' ${D}${libdir}/pkgconfig/libopusenc.pc
}

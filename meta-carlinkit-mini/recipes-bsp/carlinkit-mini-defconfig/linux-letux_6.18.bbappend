FILESEXTRAPATHS:prepend:clk-mini-ultra := "${THISDIR}/files:"

SRC_URI:append:clk-mini-ultra = " \
    file://clk-mini-ultra-add-x1600-ax1800m-dtb.patch \
    file://clk-mini-ultra-reduce-appended-dtb-padding.patch \
"

SRC_URI:append:clk-mini-ultra-nor = " \
    file://clk-mini-ultra-nor_defconfig \ 
    file://arch/mips/boot/dts/ingenic/carlinkit-mini-ultra-nor.dts \
"

SRC_URI:append:clk-mini-ultra-nor-recov = " \
    file://clk-mini-ultra-nor_defconfig \ 
    file://arch/mips/boot/dts/ingenic/carlinkit-mini-ultra-nor.dts \
"

COMPATIBLE_MACHINE:append = "|clk-mini-ultra-nor|clk-mini-ultra-recov"

do_patch:append:clk-mini-ultra() {
    cp -f ${WORKDIR}/arch/mips/boot/dts/ingenic/*.dts ${S}/arch/mips/boot/dts/ingenic/ || true
}

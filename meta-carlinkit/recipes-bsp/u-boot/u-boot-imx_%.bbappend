FILESEXTRAPATHS:append := "${THISDIR}/files:"
INSANE_SKIP:${PN} += "patch-fuzz"

SRC_URI += "file://configs/imx6ul-c2a_defconfig.in"
SRC_URI += "file://arch/arm/dts/imx6ul-c2a.dts"

SRC_URI += "file://board/freescale/mx6ul_c2a/Kconfig"
SRC_URI += "file://board/freescale/mx6ul_c2a/Makefile"
SRC_URI += "file://board/freescale/mx6ul_c2a/imximage.cfg"
SRC_URI += "file://board/freescale/mx6ul_c2a/mx6ul_c2a.c"

SRC_URI += "file://include/configs/mx6ul_c2a.h.in"

SRC_URI += "file://0001-Register-custom-C2A-board.patch"

install_c2a_board() {
    bbwarn "do_patch is now"

    install -Dm 0755 ${WORKDIR}/arch/arm/dts/imx6ul-c2a.dts ${S}/arch/arm/dts/imx6ul-c2a.dts
    install -Dm 0755 ${WORKDIR}/configs/imx6ul-c2a_defconfig.in ${S}/configs/imx6ul-c2a_defconfig

    install -Dm 0755 ${WORKDIR}/board/freescale/mx6ul_c2a/Kconfig ${S}/board/freescale/mx6ul_c2a/Kconfig
    install -Dm 0755 ${WORKDIR}/board/freescale/mx6ul_c2a/imximage.cfg ${S}/board/freescale/mx6ul_c2a/imximage.cfg
    install -Dm 0755 ${WORKDIR}/board/freescale/mx6ul_c2a/mx6ul_c2a.c ${S}/board/freescale/mx6ul_c2a/mx6ul_c2a.c
    install -Dm 0755 ${WORKDIR}/board/freescale/mx6ul_c2a/Makefile ${S}/board/freescale/mx6ul_c2a/Makefile
}

install_c2a_board:append:imx6ul-c2a() {
    bbwarn "Injecting imx6ul-c2a uboot defconfig!"

    sed -e "s|@KERNEL_OFFSET@|${C2A_KERNEL_OFFSET_HEX}|" \
        -e "s|@KERNEL_SIZE@|${C2A_KERNEL_SIZE_HEX}|" \
        ${WORKDIR}/include/configs/mx6ul_c2a.h.in > ${S}/include/configs/mx6ul_c2a.h
}

do_patch:append() {
    bb.build.exec_func('install_c2a_board', d)
}

do_patch[vardeps] += "C2A_KERNEL_OFFSET_HEX C2A_KERNEL_SIZE_HEX"

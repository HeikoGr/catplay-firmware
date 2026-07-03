FILESEXTRAPATHS:prepend:imx6ul-c2a := "${THISDIR}/files:"
SRC_URI:append:imx6ul-c2a = " file://imx6ul-c2a_defconfig"
KBUILD_DEFCONFIG:forcevariable:imx6ul-c2a = ""

COMPATIBLE_MACHINE:append = "|imx6ul-c2a"

FILESEXTRAPATHS:append := "${THISDIR}/files:"

SRC_URI += "file://imx6ul-c2a.dtsi"
SRC_URI += "file://imx6ul-c2a-flash.dtsi"
SRC_URI += "file://imx6ul-c2a-rtl8822cs.dts"
SRC_URI += "file://imx6ul-c2a-bcm4358.dts"
SRC_URI += "file://imx6ul-c2a-bcm4335.dts"

DT_FILES:imx6ul-c2a-rtl8822cs = "imx6ul-c2a-rtl8822cs.dts"
DT_FILES:imx6ul-c2a-bcm4358 = "imx6ul-c2a-bcm4358.dts"
DT_FILES:imx6ul-c2a-bcm4335 = "imx6ul-c2a-bcm4335.dts"

inherit devicetree_c2a

DT_INCLUDE:append = " ${STAGING_KERNEL_DIR}/arch/${ARCH}/boot/dts/nxp/imx"
C2A_DTC_INTS:append = " CARLINKIT_UBOOT_OFFSET_HEX CARLINKIT_UBOOT_SIZE_HEX CARLINKIT_UBOOT_CSF_OFFSET_HEX CARLINKIT_UBOOT_CSF_SIZE_HEX "
C2A_DTC_INTS:append = " CARLINKIT_UBOOT_ENV_OFFSET_HEX CARLINKIT_UBOOT_ENV_SIZE_HEX"
COMPATIBLE_MACHINE = "|imx6ul-c2a"

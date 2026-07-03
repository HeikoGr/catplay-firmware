DEPENDS:imx6ul-c2a += "carlinkit-uboot-native"
EXTRA_IMAGEDEPENDS:imx6ul-c2a  += "carlinkit-uboot-native"

CARLINKIT_WIC_UBOOT_PATH = "${RECIPE_SYSROOT_NATIVE}${libexecdir}/u-boot-carlinkit.imx.nohdr"
CARLINKIT_WIC_UBOOT_CSF_PATH = "${RECIPE_SYSROOT_NATIVE}${libexecdir}/u-boot-carlinkit.csf"


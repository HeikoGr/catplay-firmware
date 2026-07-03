FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"
SRC_URI:remove = "file://hostapd-Update-defconfig-based-on-v2.11-version.patch"

DEPENDS:remove = "openssl"
DEPENDS:append = " wolfssl"

DEPENDS:append = " libtommath"

INITSCRIPT_PARAMS = "disable"

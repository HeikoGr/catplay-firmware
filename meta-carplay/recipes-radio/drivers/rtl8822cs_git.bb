SRCREV = "3603aaf4862637c1558cb900bda6e778e052fd74"
PV = "1.0+git${SRCPV}"
PR = "r2"

SRC_URI = "git://github.com/friendlyarm/rtl8822cs.git;branch=nanopi-r2;protocol=https"

S = "${WORKDIR}/git"

REALTEK_MODULE ?= "88x2cs"
REALTEK_CFLAGS = "-DCONFIG_LITTLE_ENDIAN \
                  -DCONFIG_IOCTL_CFG80211 \
                  -DRTW_USE_CFG80211_STA_EVENT \
                  -DCONFIG_RTW_IOCTL_SET_COUNTRY \
                  -DCONFIG_CONCURRENT_MODE \
"
REALTEK_TARGET ?= "RTL8822CS"

require realtek.inc
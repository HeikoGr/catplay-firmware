PACKAGECONFIG = "openssl"
PACKAGECONFIG[openssl] = ",,wolfssl"

FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"
 
SRC_URI += "file://defconfig"

DEPENDS += "libtommath"

do_configure () {
	${MAKE} -C wpa_supplicant clean
	cat ${WORKDIR}/defconfig > wpa_supplicant/.config

	# if ${@ bb.utils.contains('PACKAGECONFIG', 'openssl', 'true', 'false', d) }; then
	# 	echo 'CONFIG_TLS=openssl' >>wpa_supplicant/.config
	# elif ${@ bb.utils.contains('PACKAGECONFIG', 'gnutls', 'true', 'false', d) }; then
	# 	echo 'CONFIG_TLS=gnutls' >>wpa_supplicant/.config
    #     sed -i -e 's/\(^CONFIG_DPP=\)/#\1/' \
    #            -e 's/\(^CONFIG_EAP_PWD=\)/#\1/' \
    #            -e 's/\(^CONFIG_SAE=\)/#\1/' wpa_supplicant/.config
	# fi

	# For rebuild
	rm -f wpa_supplicant/*.d wpa_supplicant/dbus/*.d
}

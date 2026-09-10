require recipes-connectivity/openssl/openssl_3.5.7.bb

# The upstream recipe keeps its patches next to openssl_*.bb.  This recipe is
# deliberately thin so an OE LTS update also updates the common OpenSSL build
# integration instead of leaving a version-specific copy behind.
FILESEXTRAPATHS:prepend := "${COREBASE}/meta/recipes-connectivity/openssl/openssl:${COREBASE}/meta/recipes-connectivity/openssl/files:"
S = "${UNPACKDIR}/openssl-${PV}"

# Only the target-side, static libraries are needed by catplay and
# catplay-bench.  PACKAGECONFIG's disabled sides turn off TLS 1.0 and 1.1.
BBCLASSEXTEND = ""
PACKAGECONFIG = ""
DISABLE_STATIC = ""
EXTRA_OECONF:append:class-target = " no-shared no-module no-legacy"

# The full openssl recipe owns these globally named runtime packages.  Slim
# installs no runtime SSL objects, so retain only its namespaced default,
# development and static-development packages.
PACKAGES:remove = "libcrypto libssl openssl-conf ${PN}-engines ${PN}-misc ${PN}-ossl-module-legacy ${PN}-ossl-module-fips"

do_install:append:class-target() {
    rm -f ${D}${bindir}/openssl ${D}${bindir}/c_rehash
    rm -rf ${D}${libdir}/ssl-3
    rm -rf ${D}${libdir}/engines-3
    rm -rf ${D}${libdir}/ossl-modules
    rm -rf ${D}${sysconfdir}/ssl
    rm -rf ${D}${mandir}
    rm -f ${D}${libdir}/libcrypto.so*
    rm -f ${D}${libdir}/libssl.so*
    if [ -d ${D}${bindir} ]; then
        rmdir --ignore-fail-on-non-empty ${D}${bindir}
    fi
    if [ -d ${D}${sysconfdir} ]; then
        rmdir --ignore-fail-on-non-empty ${D}${sysconfdir}
    fi
}

PKG:${PN}-staticdev:class-target = "openssl-slim-static"

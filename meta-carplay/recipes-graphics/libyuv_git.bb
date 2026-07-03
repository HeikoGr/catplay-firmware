DESCRIPTION = "Google libyuv library"
HOMEPAGE = "https://chromium.googlesource.com/libyuv/libyuv"
LICENSE = "BSD-3-Clause"
LIC_FILES_CHKSUM = "file://LICENSE;md5=464282cfb405b005b9637f11103a7325"

SRC_URI = "git://chromium.googlesource.com/libyuv/libyuv.git;protocol=https;branch=main"
SRCREV = "917276084a49be726c90292ff0a6b0a3d571a6af"

S = "${WORKDIR}/git"

inherit cmake

EXTRA_OECMAKE = "-DTESTING=OFF"

do_install:append() {
    rm -f ${D}${libdir}/libyuv.so
}

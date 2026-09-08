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

# This libyuv revision enables its NEON objects solely from
# CMAKE_SYSTEM_PROCESSOR matching /^arm/; it does not consume a
# LIBYUV_DISABLE_NEON CMake option. Use a neutral CMake processor name to keep
# those objects out, and also guard the C/C++ headers explicitly.
EXTRA_OECMAKE:append:armv5 = " -DCMAKE_SYSTEM_PROCESSOR=generic -DLIBYUV_DISABLE_NEON=ON"
CFLAGS:append:armv5 = " -DLIBYUV_DISABLE_NEON"
CXXFLAGS:append:armv5 = " -DLIBYUV_DISABLE_NEON"

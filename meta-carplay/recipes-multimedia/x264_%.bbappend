FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"

EXTRA_OECONF:append = " --disable-shared --enable-static"
EXTRA_OECONF:remove = "--enable-shared"

SRC_URI = "git://code.videolan.org/videolan/x264;branch=master;protocol=https \
           file://don-t-default-to-cortex-a9-with-neon.patch \
           file://Fix-X32-build-by-disabling-asm.patch \
           "

SRC_URI:append = " file://x264-lowmem.patch"
SRCREV = "0480cb05fa188d37ae87e8f4fd8f1aea3711f7ee"
EXTRA_OEMAKE:append = " X264_LOW_MEMORY_BUILD=1"

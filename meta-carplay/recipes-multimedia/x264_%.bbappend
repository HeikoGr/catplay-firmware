FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"

# Catplay links libx264 directly and does not use the x264 CLI's libavformat
# input support.  Keeping OE-Core's default "ffmpeg" PACKAGECONFIG here pulls
# full FFmpeg into Catplay's sysroot beside the intentionally minimal static
# ffmpeg-mini provider, and both export the same libav* headers.
PACKAGECONFIG:remove = "ffmpeg"

EXTRA_OECONF:append = " --disable-shared --enable-static"
EXTRA_OECONF:remove = "--enable-shared"

SRC_URI:append = " file://x264-lowmem.patch"
EXTRA_OEMAKE:append = " X264_LOW_MEMORY_BUILD=1"

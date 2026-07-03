SUMMARY = "Minimal FFmpeg build (AAC/Opus/PCM only, static, Rust-bitcode-ready LTO)"
HOMEPAGE = "https://ffmpeg.org/"
LICENSE = "LGPL-2.1-or-later"
LIC_FILES_CHKSUM = "file://COPYING.LGPLv2.1;md5=eed22b3456132611e3d4aa7a7ec64dac"

SRC_URI = "https://ffmpeg.org/releases/ffmpeg-${PV}.tar.gz"
SRC_URI[sha256sum] = "cce1136d38c389e6baaa452d6babc384cb2d3a9406ebe48c36a48f3ee115d8df"

SRC_URI += "file://0001-fix-clang-lto-build-on-mips.patch"
SRC_URI += "file://0002-fix-mips-macro-conflict.patch"
SRC_URI += "file://0003-ingenic-x1600-flags.patch"

PV = "8.0"

S = "${WORKDIR}/ffmpeg-${PV}"

DEPENDS = "zlib libfdk-aac"

# Build fails when thumb is enabled: https://bugzilla.yoctoproject.org/show_bug.cgi?id=7717
ARM_INSTRUCTION_SET:armv4 = "arm"
ARM_INSTRUCTION_SET:armv5 = "arm"
ARM_INSTRUCTION_SET:armv6 = "arm"

DEPENDS:append:x86 = " nasm-native"
DEPENDS:append:x86-64 = " nasm-native"

FFMPEG_CONF = "\
    --disable-everything \
    --enable-decoder=aac \
    --enable-decoder=opus \
    --enable-parser=aac \
    --enable-parser=opus \
    --enable-demuxer=ogg \
    \
    --disable-encoders \
    --disable-muxers \
    --disable-filters \
    --disable-programs \
    --disable-doc \
    --disable-debug \
    --disable-network \
    --disable-logging \
    \
    --enable-small \
    --enable-nonfree \
    --enable-libfdk-aac \
    --enable-static \
    --disable-shared \
    --enable-lto \
    --enable-pic \
    --enable-pthreads \
    \
    --extra-cflags='-flto=full -fvisibility=hidden -ffunction-sections -fdata-sections' \
    --extra-ldflags='-flto=full -Wl,--gc-sections' \
    \
    --disable-alsa \
    --disable-sdl2 \
    --disable-xlib \
    --disable-libxcb \
    --disable-vulkan \
    --disable-libdrm \
    --disable-iconv \
    --disable-lzma \
    --disable-zlib \
    --disable-cuda \
    --disable-cuvid \
    --disable-nvdec \
    --disable-nvenc \
    --disable-cuda-llvm \
    --disable-v4l2-m2m \
"

def cpu(d):
    for arg in (d.getVar('TUNE_CCARGS') or '').split():
        if arg.startswith('-mcpu='):
            return arg[6:]
    return 'generic'

EXTRA_OECONF = " \
    ${FFMPEG_CONF} \
    --cross-prefix=${TARGET_PREFIX} \
    \
    --ld='${CCLD}' \
    --cc='${CC}' \
    --cxx='${CXX}' \
    --arch=${TARGET_ARCH} \
    --ranlib=${RANLIB} \
    --nm=${NM} \
    --ar=${AR} \
    --strip=${STRIP} \
    --target-os='linux' \
    --enable-cross-compile \
    --extra-cflags='${CFLAGS} ${HOST_CC_ARCH}${TOOLCHAIN_OPTIONS}' \
    --extra-ldflags='${LDFLAGS}' \
    --sysroot='${STAGING_DIR_TARGET}' \
    ${EXTRA_FFCONF} \
    --libdir=${libdir} \
    --shlibdir=${libdir} \
    --datadir=${datadir}/ffmpeg \
    ${@bb.utils.contains('AVAILTUNES', 'mips32r2', '', '--disable-mipsdsp --disable-mipsdspr2', d)} \
    --cpu=${@cpu(d)} \
    --pkg-config=pkg-config \
"

EXTRA_OECONF:append:linux-gnux32 = " --disable-asm"
# --enable-pic is used and x86 assembly is not PIC on x86
EXTRA_OECONF:append:x86 = " --disable-asm"

EXTRA_OECONF += "${@bb.utils.contains('TUNE_FEATURES', 'mipsisa64r6', '--disable-mips64r2 --disable-mips32r2', '', d)}"
EXTRA_OECONF += "${@bb.utils.contains('TUNE_FEATURES', 'mipsisa64r2', '--disable-mips64r6 --disable-mips32r6', '', d)}"
EXTRA_OECONF += "${@bb.utils.contains('TUNE_FEATURES', 'mips32r2', '--disable-mips64r6 --disable-mips32r6', '', d)}"
EXTRA_OECONF += "${@bb.utils.contains('TUNE_FEATURES', 'mips32r6', '--disable-mips64r2 --disable-mips32r2', '', d)}"
EXTRA_OECONF:append:mips = " --extra-libs=-latomic --disable-mips32r5 --disable-mipsdsp --disable-mipsdspr2 \
                             --disable-loongson2 --disable-loongson3 --disable-mmi --disable-msa"
EXTRA_OECONF:append:riscv32 = " --extra-libs=-latomic --disable-rvv --disable-asm"
EXTRA_OECONF:append:armv5 = " --extra-libs=-latomic"
EXTRA_OECONF:append:powerpc = " --extra-libs=-latomic"
EXTRA_OECONF:append:armv7a = "${@bb.utils.contains('TUNE_FEATURES','neon','',' --disable-neon',d)}"
EXTRA_OECONF:append:armv7ve = "${@bb.utils.contains('TUNE_FEATURES','neon','',' --disable-neon',d)}"

PACKAGE_ARCH:ingenic-x1600 = "ingenic-x1600"
EXTRA_OECONF:append:ingenic-x1600 = " --cpu=ingenic-x1600"
CFLAGS:append:ingenic-x1600 = " -march=mips32r5 -mdsp -mdspr2"

LDFLAGS:append:x86 = "${@bb.utils.contains('DISTRO_FEATURES', 'ld-is-lld', ' -fuse-ld=bfd ', '', d)}"

EXTRA_OEMAKE = "V=1"

inherit autotools pkgconfig

do_configure() {
    export TMPDIR="${B}/tmp"
    mkdir -p ${B}/tmp
    ${S}/configure ${EXTRA_OECONF}
    sed -i -e "s,^X86ASMFLAGS=.*,& --debug-prefix-map=${S}=${TARGET_DBGSRC_DIR} --debug-prefix-map=${B}=${TARGET_DBGSRC_DIR},g" ${B}/ffbuild/config.mak
}

# patch out build host paths for reproducibility
do_compile:prepend:class-target() {
    sed -i -e "s,${WORKDIR},,g" ${B}/config.h
}

FILES:${PN} = "${libdir}/*.a"
FILES:${PN}-dev = "${includedir} ${libdir}/pkgconfig"

PACKAGES += "${PN}-examples"
FILES:${PN}-examples = "${datadir}/ffmpeg/examples"

# [HACK] This fixes ffmpeg-sys-next build crash on MIPS related to check.c 

FFMPEG_DIR = "${B}/fake-ffmpeg"

do_configure:append() {
    rm -rf "${FFMPEG_DIR}"
    mkdir -p ${FFMPEG_DIR}/include ${FFMPEG_DIR}/lib
    ln -sf ${RECIPE_SYSROOT}/usr/include/libavcodec    ${FFMPEG_DIR}/include/
    ln -sf ${RECIPE_SYSROOT}/usr/include/libavformat   ${FFMPEG_DIR}/include/
    ln -sf ${RECIPE_SYSROOT}/usr/include/libavdevice   ${FFMPEG_DIR}/include/
    ln -sf ${RECIPE_SYSROOT}/usr/include/libavutil     ${FFMPEG_DIR}/include/
    ln -sf ${RECIPE_SYSROOT}/usr/include/libswresample ${FFMPEG_DIR}/include/
    ln -sf ${RECIPE_SYSROOT}/usr/include/libswscale    ${FFMPEG_DIR}/include/
    ln -sf ${RECIPE_SYSROOT}/usr/include/libavfilter   ${FFMPEG_DIR}/include/
    ln -sf ${RECIPE_SYSROOT}/usr/lib/*av*.a   ${FFMPEG_DIR}/lib/ || true
}

do_compile:prepend() {
    export FFMPEG_DIR="${FFMPEG_DIR}"
    export CFLAGS_x86_64_unknown_linux_gnu="-fno-lto"
    export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=${RECIPE_SYSROOT} -I${RECIPE_SYSROOT}/usr/include"
}
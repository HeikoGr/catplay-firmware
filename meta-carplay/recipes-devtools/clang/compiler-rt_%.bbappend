# compiler-rt normally configures directly from OE-Core's shared LLVM source
# tree. Attaching a patch to llvm-project-source would invalidate every LLVM
# consumer, including clang and llvm themselves. Build a cheap private overlay
# for the target compiler-rt recipe instead: only runtimes and llvm-libgcc are
# copied, while the remaining top-level directories are symlinked.
C2A_LLVM_PRIVATE_SOURCE = "${WORKDIR}/c2a-llvm-project-source"
C2A_LLVM_LIBGCC_PATCH_1 := "${THISDIR}/compiler-rt/0001-llvm-libgcc-pass-target-flags-when-generating-version-script.patch"
C2A_LLVM_LIBGCC_PATCH_2 := "${THISDIR}/compiler-rt/0002-llvm-libgcc-make-builtins-install-path-relative.patch"
C2A_LLVM_LIBGCC_PATCHES := " \
    ${C2A_LLVM_LIBGCC_PATCH_1} \
    ${C2A_LLVM_LIBGCC_PATCH_2} \
"

OECMAKE_SOURCEPATH:class-target = "${C2A_LLVM_PRIVATE_SOURCE}/runtimes"
DEPENDS:append:class-target = " patch-native"

# The private source overlay is intentionally outside ${S}, so OE-Core's
# standard reproducible-build flags do not cover it.  Map it explicitly to
# prevent compiler-rt/libunwind from embedding TMPDIR through __FILE__.
DEBUG_PREFIX_MAP:append:class-target = " \
    -ffile-prefix-map=${C2A_LLVM_PRIVATE_SOURCE}=${TARGET_DBGSRC_DIR} \
    -fmacro-prefix-map=${C2A_LLVM_PRIVATE_SOURCE}=${TARGET_DBGSRC_DIR} \
"

do_configure[file-checksums] += " \
    ${C2A_LLVM_LIBGCC_PATCH_1}:True \
    ${C2A_LLVM_LIBGCC_PATCH_2}:True \
"

do_configure:prepend:class-target() {
    private_source="${C2A_LLVM_PRIVATE_SOURCE}"

    rm -rf -- "$private_source"
    install -d "$private_source"

    for source_entry in "${S}"/*; do
        entry_name="$(basename "$source_entry")"
        case "$entry_name" in
            runtimes|llvm-libgcc)
                cp -a "$source_entry" "$private_source/$entry_name"
                ;;
            *)
                ln -s "$source_entry" "$private_source/$entry_name"
                ;;
        esac
    done

    for llvm_libgcc_patch in ${C2A_LLVM_LIBGCC_PATCHES}; do
        "${STAGING_BINDIR_NATIVE}/patch" \
            -d "$private_source" -p1 \
            < "$llvm_libgcc_patch"
    done
}

# LLVM normalizes the ARM hard-float CRT filenames to the "arm" suffix, while
# OE-Core 6.0 currently creates its compatibility links with "armhf".  Point
# those links at the names actually emitted by compiler-rt.
do_install:append:class-target() {
    if [ "${HF}" = "hf" ]; then
        ln -sfn clang/${INSTALL_VER}/lib/linux/clang_rt.crtbegin-${HOST_ARCH}.o \
            ${D}${nonarch_libdir}/crtbegin.o
        ln -sfn clang/${INSTALL_VER}/lib/linux/clang_rt.crtbegin-${HOST_ARCH}.o \
            ${D}${nonarch_libdir}/crtbeginS.o
        # Clang requests crtbeginT.o for fully static executables.  LLVM uses
        # the same crtbegin implementation for the normal, PIE/shared and
        # static compatibility names.
        ln -sfn clang/${INSTALL_VER}/lib/linux/clang_rt.crtbegin-${HOST_ARCH}.o \
            ${D}${nonarch_libdir}/crtbeginT.o
        ln -sfn clang/${INSTALL_VER}/lib/linux/clang_rt.crtend-${HOST_ARCH}.o \
            ${D}${nonarch_libdir}/crtend.o
        ln -sfn clang/${INSTALL_VER}/lib/linux/clang_rt.crtend-${HOST_ARCH}.o \
            ${D}${nonarch_libdir}/crtendS.o
    fi
}

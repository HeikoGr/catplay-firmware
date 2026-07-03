inherit image-artifact-names
inherit c2a-artifact-names

C2A_BUNDLE_NAME ?= "${MACHINE}"
C2A_BUNDLE_DIR ?= "${TOPDIR}/tmp-c2a/${C2A_BUNDLE_NAME}"

C2A_BUNDLE_EXTRA_DEPENDS ?= ""
C2A_BUNDLE_ARTIFACTS ?= ""

do_c2a_bundle[depends] += "${C2A_SYSTEM_IMAGE}:do_image_complete virtual/kernel:do_deploy"
do_c2a_bundle[depends] += "${C2A_INITRAMFS_IMAGE}:do_image_complete"
do_c2a_bundle[depends] += "${C2A_BUNDLE_EXTRA_DEPENDS}"
# Don't clean; for now we want to allow multiple machines to populate a shared bundle...
#do_c2a_bundle[cleandirs] += "${C2A_BUNDLE_DIR}"
do_c2a_bundle[nostamp] = "1"
do_build[recrdeptask] += "do_c2a_bundle"
addtask c2a_bundle before do_build after do_compile

do_c2a_bundle() {
    bundle_dir="${C2A_BUNDLE_DIR}"
    install -d "${bundle_dir}"

    if [ -z "${C2A_WIC_INITRAMFS_EXT}" ]; then
        bbfatal "C2A_WIC_INITRAMFS_EXT is empty. Set C2A_INITRAMFS_FSTYPES or INITRAMFS_FSTYPES."
    fi

    for artifact in ${C2A_BUNDLE_ARTIFACTS}; do
        src="${artifact%%,*}"
        rest="${artifact#*,}"
        rel_dst="${rest%%,*}"
        required="${rest#*,}"

        if [ "${rest}" = "${artifact}" ] || [ "${required}" = "${rest}" ]; then
            bbfatal "Invalid C2A_BUNDLE_ARTIFACTS entry: ${artifact} (expected src,rel_dst,required)"
        fi

        case "${required}" in
            1|true|yes)
                ;;
            0|false|no)
                bbnote "Skipping disabled bundle artifact: ${src}"
                continue
                ;;
            *)
                bbfatal "Invalid required flag in C2A_BUNDLE_ARTIFACTS entry: ${artifact} (expected 0/1, true/false, or yes/no)"
                ;;
        esac

        dst="${bundle_dir}/${rel_dst}"
        install -d "$(dirname "${dst}")"

        if [ -f "${src}" ]; then
            install -m 0644 "${src}" "${dst}"
        else
            bbfatal "Required bundle artifact not found: ${src}"
        fi
    done
}

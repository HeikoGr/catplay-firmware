IMAGE_EROFS_COMPRESS_HINTS ?= ""
IMAGE_EROFS_PCLUSTER ?= "32768"
# ztailpacking
IMAGE_EROFS_FEATURES ?= "dedupe fragments force-inode-compact"

EROFS_COMPRESS_HINTS ?= "${IMAGE_EROFS_COMPRESS_HINTS}"
EROFS_PCLUSTER ?= "${IMAGE_EROFS_PCLUSTER}"
EROFS_FEATURES ?= "${IMAGE_EROFS_FEATURES}"

python erofs_prepare_mkfs_opts() {
    import os
    import shlex

    base_algorithm = d.getVar("EROFS_BASE_ALGORITHM")
    image_type = d.getVar("EROFS_IMAGE_TYPE")
    hints_text = d.getVar("EROFS_COMPRESS_HINTS") or ""
    max_pcluster = int(d.getVar("EROFS_PCLUSTER"), 0)
    extra_imagecmd = d.getVar("EXTRA_IMAGECMD") or ""
    features_raw = d.getVar("EROFS_FEATURES") or ""

    hints_file = os.path.join(d.getVar("T"), f"{d.getVar('IMAGE_NAME')}.{image_type}.compress-hints")
    image_path = os.path.join(d.getVar("IMGDEPLOYDIR"), f"{d.getVar('IMAGE_NAME')}.{image_type}")
    image_rootfs = d.getVar("IMAGE_ROOTFS")

    for path in (hints_file,):
        try:
            os.unlink(path)
        except FileNotFoundError:
            pass

    algorithms = [base_algorithm]
    algorithm_indexes = {base_algorithm: 0}
    rewritten = []

    if hints_text.strip():
        for raw_line in hints_text.splitlines():
            line = raw_line.strip()
            if not line or line.startswith("#"):
                continue

            try:
                parts = shlex.split(raw_line, comments=False, posix=True)
            except ValueError as exc:
                bb.fatal(f"Invalid EROFS_COMPRESS_HINTS line: {raw_line}: {exc}")

            if len(parts) < 3:
                bb.fatal(f"Invalid EROFS_COMPRESS_HINTS line: {raw_line}")

            pcluster, algorithm = parts[0], parts[1]
            regex = " ".join(parts[2:])
            try:
                pcluster_value = int(pcluster, 0)
            except ValueError:
                bb.fatal(f"Invalid EROFS_COMPRESS_HINTS pcluster value: {pcluster}")

            if pcluster_value <= 0:
                bb.fatal(f"Invalid EROFS_COMPRESS_HINTS pcluster value: {pcluster}")

            if algorithm not in algorithms:
                algorithm_indexes[algorithm] = len(algorithms)
                algorithms.append(algorithm)

            rewritten.append(f"{pcluster} {algorithm_indexes[algorithm]} {regex}")
            max_pcluster = max(max_pcluster, pcluster_value)

        with open(hints_file, "w", encoding="utf-8") as hints_out:
            hints_out.write("\n".join(rewritten))
            hints_out.write("\n")

    features = []
    for token in shlex.split(features_raw):
        for feature in token.split(","):
            feature = feature.strip()
            if feature and feature not in features:
                features.append(feature)

    mkfs_opts = ["-z", ":".join(algorithms)]

    if rewritten:
        if not any(
            token == "-C" or
            token.startswith("-C") or
            token == "--pclustersize" or
            token.startswith("--pclustersize=")
            for token in shlex.split(extra_imagecmd)
        ):
            mkfs_opts.extend(["-C", str(max_pcluster)])
        mkfs_opts.append(f"--compress-hints={hints_file}")

    if features:
        mkfs_opts.append(f"-E{','.join(features)}")

    d.setVar("EROFS_MKFS_OPTS", shlex.join(mkfs_opts))

    if rewritten:
        bb.warn("EROFS compress hints file content:\n%s" % "\n".join(rewritten))
    else:
        bb.warn("EROFS compress hints file content: <empty>")

    mkfs_cmd = ["mkfs.erofs"] + mkfs_opts
    mkfs_cmd.extend(shlex.split(extra_imagecmd))
    mkfs_cmd.extend([image_path, image_rootfs])
    bb.warn("EROFS mkfs command line: %s" % shlex.join(mkfs_cmd))
}

oe_mkerofs() {
    image="$1"
    rootfs="$2"
    shift 2

    eval "set -- ${EROFS_MKFS_OPTS} \"\$@\" \"\${image}\" \"\${rootfs}\""

    set +e
    mkfs_version_output="$(mkfs.erofs -V 2>&1)"
    mkfs_version_status=$?
    set -e
    if [ -n "${mkfs_version_output}" ]; then
        bbwarn "EROFS mkfs.erofs -V output:\n${mkfs_version_output}"
    else
        bbwarn "EROFS mkfs.erofs -V output: <empty>"
    fi
    if [ "${mkfs_version_status}" -ne 0 ]; then
        bbfatal "mkfs.erofs -V exited with status ${mkfs_version_status}"
    fi

    set +e
    mkfs.erofs "$@"
    mkfs_status=$?
    set -e
    bbwarn "mkfs.erofs exited with status ${mkfs_status}"
    if [ "${mkfs_status}" -ne 0 ]; then
        exit "${mkfs_status}"
    fi

    set +e
    dump_output="$(dump.erofs -S "${image}" 2>&1)"
    dump_status=$?
    set -e
    if [ -n "${dump_output}" ]; then
        bbwarn "EROFS dump.erofs -S output for ${image}:\n${dump_output}"
    else
        bbwarn "EROFS dump.erofs -S output for ${image}: <empty>"
    fi
    if [ "${dump_status}" -ne 0 ]; then
        bbwarn "dump.erofs -S exited with status ${dump_status}"
    fi
}

IMAGE_CMD:erofs-lz4hc:forcevariable = "oe_mkerofs ${IMGDEPLOYDIR}/${IMAGE_NAME}.erofs-lz4hc ${IMAGE_ROOTFS} ${EXTRA_IMAGECMD}"
IMAGE_CMD:erofs-lz4:forcevariable = "oe_mkerofs ${IMGDEPLOYDIR}/${IMAGE_NAME}.erofs-lz4 ${IMAGE_ROOTFS} ${EXTRA_IMAGECMD}"

do_image_erofs_lz4[prefuncs] += "erofs_prepare_mkfs_opts"
do_image_erofs_lz4hc[prefuncs] += "erofs_prepare_mkfs_opts"
do_image_erofs_lz4[vardeps] += "EROFS_COMPRESS_HINTS EROFS_PCLUSTER EROFS_BASE_ALGORITHM EROFS_IMAGE_TYPE EROFS_FEATURES EXTRA_IMAGECMD EROFS_MKFS_OPTS"
do_image_erofs_lz4hc[vardeps] += "EROFS_COMPRESS_HINTS EROFS_PCLUSTER EROFS_BASE_ALGORITHM EROFS_IMAGE_TYPE EROFS_FEATURES EXTRA_IMAGECMD EROFS_MKFS_OPTS"

EROFS_BASE_ALGORITHM:task-image-erofs-lz4 = "lz4"
EROFS_BASE_ALGORITHM:task-image-erofs-lz4hc = "lz4hc,12"
EROFS_IMAGE_TYPE:task-image-erofs-lz4 = "erofs-lz4"
EROFS_IMAGE_TYPE:task-image-erofs-lz4hc = "erofs-lz4hc"

EROFS_HELPER_LOADED = "1"

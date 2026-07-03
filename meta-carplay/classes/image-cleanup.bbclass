ROOTFS_POSTPROCESS_COMMAND += " cleanup_image_files; "
do_rootfs[vardeps] += "IMAGE_CLEANUP_TARGETS IMAGE_CLEANUP_VARIANT"

python cleanup_image_files() {
    import os
    import glob
    import shutil

    variant = d.getVar("IMAGE_CLEANUP_VARIANT") or ""
    rootfs = d.getVar("IMAGE_ROOTFS")
    garbage = d.getVar("IMAGE_CLEANUP_TARGETS") or ""
    variant_garbage = d.getVarFlag("IMAGE_CLEANUP_TARGETS", variant) if variant else ""

    bb.warn(f"📦 NOTE: Extra image cleanup variant is '{variant}'")

    paths = garbage.split() + variant_garbage.split()

    for path_pattern in paths:
        if not path_pattern or path_pattern == "/" or not rootfs:
            bb.warn(f"❌ Skipping dangerous or empty entry: {path_pattern}")
            continue

        full_pattern = os.path.join(rootfs, path_pattern.lstrip("/"))
        matches = glob.glob(full_pattern)

        if not matches:
            bb.warn(f"📦 No matches found for: {path_pattern}")

        for fullpath in matches:
            if os.path.commonpath([rootfs, fullpath]) != rootfs:
                bb.warn(f"❌ Skipping suspicious path: {path_pattern}")
                continue

            relpath = os.path.relpath(fullpath, rootfs)
            bb.warn(f"✅ Removing from image /{relpath}")
            try:
                if os.path.isdir(fullpath):
                    shutil.rmtree(fullpath)
                else:
                    os.remove(fullpath)
            except Exception as e:
                bb.warn(f"❌ Failed to remove {fullpath}: {e}")
}

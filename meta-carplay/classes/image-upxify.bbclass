ROOTFS_POSTPROCESS_COMMAND += " upx_pack_image_files; "
EXTRA_IMAGEDEPENDS = "upx-native"
#DEPENDS += "upx-native"

do_rootfs[depends] += "upx-native:do_populate_sysroot"
do_rootfs[vardeps] += "IMAGE_UPX_TARGETS"

UPX_WORKERS ??= "${@str(os.cpu_count() or 1)}"

python upx_pack_image_files() {
    import concurrent.futures
    import os
    import shutil
    import subprocess
    import tempfile

    bb.warn("Running UPX on image binaries...")

    image_rootfs = d.getVar("IMAGE_ROOTFS")
    targets = (d.getVar("IMAGE_UPX_TARGETS") or "").split()
    tmp_base = d.getVar("T")
    upx = os.path.join(d.getVar("STAGING_BINDIR_NATIVE"), "upx")
    workers = int(d.getVar("UPX_WORKERS") or (os.cpu_count() or 1))

    jobs = []
    for target in targets:
        fpath = os.path.join(image_rootfs, target.lstrip("/"))
        if os.path.isfile(fpath):
            bb.warn("📦 UPX packing -> %s" % target)
            jobs.append((target, fpath))

    if not jobs:
        return

    def pack(job):
        target, fpath = job
        tmpdir = tempfile.mkdtemp(prefix="upx.", dir=tmp_base)
        try:
            infile = os.path.join(tmpdir, "in")
            outfile = os.path.join(tmpdir, "out")
            shutil.copy2(fpath, infile)
            result = subprocess.run(
                [upx, "--lzma", "-o", outfile, infile],
                cwd=tmpdir,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                check=False,
            )
            if result.returncode != 0:
                return (target, False, result.stdout)
            shutil.copy2(outfile, fpath)
            os.chmod(fpath, 0o755)
            return (target, True, result.stdout)
        finally:
            shutil.rmtree(tmpdir, ignore_errors=True)

    failures = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
        future_to_target = {executor.submit(pack, job): job[0] for job in jobs}
        for future in concurrent.futures.as_completed(future_to_target):
            target = future_to_target[future]
            try:
                packed_target, ok, output = future.result()
            except Exception as exc:
                failures.append("%s: %s" % (target, exc))
                continue
            if ok:
                bb.warn("✅ UPX success for %s" % packed_target)
            else:
                failures.append("%s: %s" % (packed_target, output))

    if failures:
        bb.fatal("❌ UPX failed:\n%s" % "\n".join(failures))
}

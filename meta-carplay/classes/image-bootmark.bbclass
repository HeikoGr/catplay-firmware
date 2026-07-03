ROOTFS_POSTPROCESS_COMMAND += "inject_initscript_markers; "

python inject_initscript_markers() {
    import os, glob, stat

    rootfs = d.getVar("IMAGE_ROOTFS")
    dirs = [
        os.path.join(rootfs, "etc/init.d", "*"),
    ]

    for pattern in dirs:
        for script in glob.glob(pattern):
            if script.endswith("etc/init.d/c2a-overlay-init"):
                continue
        
            if os.path.islink(script):
                continue

            with open(script, "r") as f:
                lines = f.readlines()

            if any("/dev/kmsg" in line for line in lines[:5]):
                continue

            name = os.path.basename(script)
            marker_start = f'echo "INIT START {name}" > /dev/kmsg || true\n'
            marker_end   = f'echo "INIT END   {name}" > /dev/kmsg || true\n'

            if lines and lines[0].startswith("#!"):
                new = [lines[0], marker_start] + lines[1:] + [marker_end]
            else:
                new = [marker_start] + lines + [marker_end]

            with open(script, "w") as f:
                f.writelines(new)

            st = os.stat(script)
            os.chmod(script, st.st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
}

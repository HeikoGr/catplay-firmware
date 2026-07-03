C2A_KERNEL_EXTRA_CONFIG := ""

addtask c2a_kernel_extra_config after do_kernel_configme before do_configure

python () {
    # Any change in C2A_KERNEL_EXTRA_CONFIG must invalidate the kernel build
    # path, not only the small helper task.
    for task in ("do_kernel_configme", "do_configure", "do_compile"):
        d.appendVarFlag(task, "vardeps", " C2A_KERNEL_EXTRA_CONFIG")
}

python do_c2a_kernel_extra_config() {
    cfg = d.getVar("C2A_KERNEL_EXTRA_CONFIG")
    out = d.expand("${B}/.config")

    try:
        with open(out, "r") as f:
            existing = f.read()
    except FileNotFoundError:
        existing = ""

    keys = []
    if cfg:
        for line in cfg.splitlines():
            line = line.strip()
            if not line or "=" not in line:
                continue
            key = line.split("=", 1)[0].strip()
            if key.startswith("CONFIG_"):
                keys.append(key)

    if existing and keys:
        cleaned = []
        for line in existing.splitlines():
            stripped = line.strip()
            drop = False

            for key in keys:
                if stripped.startswith(f"{key}="):
                    drop = True
                    break
                if stripped == f"# {key} is not set":
                    drop = True
                    break

            if not drop:
                cleaned.append(line)

        existing = "\n".join(cleaned).rstrip()

    if cfg:
        if existing:
            existing += "\n\n"
        existing += f"{cfg}\n"
    else:
        bb.warn("C2A_KERNEL_EXTRA_CONFIG empty; nothing to inject")
        if existing:
            existing += "\n"

    with open(out, "w") as f:
        f.write(existing)
}
do_c2a_kernel_extra_config[vardeps] += "C2A_KERNEL_EXTRA_CONFIG"

python () {
    flags = d.getVarFlags("C2A_KERNEL_EXTRA_CONFIG") or {}
    lines = []

    for k, v in sorted(flags.items()):
        if not k.startswith("CONFIG_"):
            bb.fatal(
                f"C2A_KERNEL_EXTRA_CONFIG[{k}] is invalid; "
                f"only CONFIG_* keys are allowed"
            )

        if v not in ("y", "m", "n") and not v.startswith('"'):
            bb.warn(
                f"C2A_KERNEL_EXTRA_CONFIG[{k}] has unusual value '{v}'"
            )

        lines.append(f"{k}={v}")

    if lines:
        d.setVar("C2A_KERNEL_EXTRA_CONFIG", "\n".join(lines))
}

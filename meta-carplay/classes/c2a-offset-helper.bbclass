C2A_OFFSET_HELPER_VARS ?= ""

python __anonymous() {
    def parse_size(val):
        val = val.strip().lower()
        if val.startswith("0x"):
            return int(val, 16)
        elif val.endswith("k"):
            return int(val[:-1]) * 1024
        elif val.endswith("m"):
            return int(val[:-1]) * 1024 * 1024
        elif val.endswith("g"):
            return int(val[:-1]) * 1024 * 1024 * 1024
        else:
            return int(val)

    l = d.getVar("C2A_OFFSET_HELPER_VARS").split()
    bb.warn(f"Using offset helper list: {l}")

    skipped = []
    for name in l:
        val = d.getVar(name)
        if val:
            try:
                parsed = parse_size(val)
                d.setVar(name + "_BYTES", str(parsed))
                d.setVar(name + "_HEX", hex(parsed))
            except Exception as e:
                bb.fatal(f"❌ Failed to parse {name} = '{val}': {e}")
        else:
            skipped += [name]

    if skipped:
            bb.warn(f"❌ Skipping vars {skipped} for offset helper, undefined in current context!")
}

C2A_OFFSET_HELPER_ENABLED = "1"

INSANE_SKIP:${PN} += "installed-vs-shipped"
C2A_FIRMWARE_STAGE ?= "linux-firmware"
DEPENDS += "${C2A_FIRMWARE_STAGE}"

do_install:prepend() {
    bbwarn "Staging linux-firmware(${C2A_FIRMWARE_STAGE}) now"
    mkdir -p ${D}/lib/firmware
    cp -r ${RECIPE_SYSROOT}/lib/firmware/* ${D}/lib/firmware/
}

python __anonymous () {
    mods = d.getVar('C2A_FIRMWARE_STAGE').split()
    for m in mods:
        d.appendVarFlag('do_install', 'depends', f' {m}:do_populate_sysroot')
}

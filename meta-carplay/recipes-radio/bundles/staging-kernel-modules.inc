INSANE_SKIP:${PN} += "installed-vs-shipped"
C2A_MODULES_STAGE ?= ""
DEPENDS += "virtual/kernel ${C2A_MODULES_STAGE}"

do_install:prepend() {
    bbwarn "Staging linux kernel modules now"
    mkdir -p ${D}/lib/modules
    cp -r ${RECIPE_SYSROOT}/lib/modules/* ${D}/lib/modules/
}

python __anonymous () {
    mods = d.getVar('C2A_MODULES_STAGE').split()
    for m in mods:
        d.appendVarFlag('do_install', 'depends', f' {m}:do_populate_sysroot')
}

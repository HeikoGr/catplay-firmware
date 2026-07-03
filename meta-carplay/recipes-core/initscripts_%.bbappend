MASKED_SCRIPTS_SYSVINIT = "urandom populate-volatile.sh mountnfs.sh read-only-rootfs-hook.sh mountall.sh banner.sh checkroot.sh bootmisc.sh sysfs.sh hostname.sh alignment.sh devpts.sh dmesg.sh"

do_install:append () {
    if ! type systemctl >/dev/null 2>&1; then
        for SERVICE in ${MASKED_SCRIPTS_SYSVINIT}; do
            if [ -n "${D}" ]; then
                update-rc.d -f -r ${D} ${SERVICE} remove || true
            else
                update-rc.d -f ${SERVICE} remove || true
            fi
            bbwarn "Masking initscript ${SERVICE} (sysvinit)"
        done
    fi
}

RDEPENDS:${PN} += "c2a-overlay-init"
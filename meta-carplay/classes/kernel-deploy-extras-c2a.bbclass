do_deploy:append() {
    VMLINUX_PATH="${B}/arch/arm/boot/vmlinux"
    DEFCONFIG_PATH="${B}/.config"

    if [ -e "${VMLINUX_PATH}" ]; then
        install -Dm644 "${VMLINUX_PATH}" "${DEPLOYDIR}/linux-vmlinux-${MACHINE}-${PV}-${PR}"
        gzip -9 "${DEPLOYDIR}/linux-vmlinux-${MACHINE}-${PV}-${PR}"
    else
        bbwarn "vmlinux not found at ${VMLINUX_PATH}, skipping deploy"
    fi

    install -Dm644 "${DEFCONFIG_PATH}" "${DEPLOYDIR}/linux-defconfig-${MACHINE}-${PV}-${PR}.txt"
}

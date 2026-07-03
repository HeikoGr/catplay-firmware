FILESEXTRAPATHS:append := "${THISDIR}/files:"
INSANE_SKIP:${PN} += "patch-fuzz"

SRC_URI:append = " file://0001-Fix-fastboot-on-usbotg2-on-iMX6-platform.patch"
SRC_URI:append = " file://0002-Whitelist-mx25l12805d-and-mx25u12835f-for-QSPI-support.patch"
SRC_URI:append = " file://0003-usb-gadget-skip-first-ChipIdea-UDC-on-C2A.patch"

inherit c2a-offset-helper

do_compile:append() {
    clean_machine=$(echo "${UBOOT_MACHINE}" | sed 's/ //g')
    file_path="${B}/${clean_machine}/u-boot.imx"
    size=$(stat -c%s "$file_path")

    effective_size=$(expr "$size") # + 1024)

    if [ "x$C2A_UBOOT_SIZE_ENFORCE" == "x1" ]; then
        if [ -n "$C2A_UBOOT_SIZE_BYTES" ]; then
            uboot_size_dec=$(printf "%d" "$C2A_UBOOT_SIZE_BYTES")
            if [ "$effective_size" -gt "$uboot_size_dec" ]; then
                bbfatal "u-boot too big: ${effective_size} bytes > ${uboot_size_dec} bytes" # (including 0x400 header)"
            fi
        fi
    fi
}

do_deploy:append() {
    UBOOT_MACHINE_CLEAN=$(echo "${UBOOT_MACHINE}" | sed 's/ //g')
    UBOOT_ELF_PATH="${B}/${UBOOT_MACHINE_CLEAN}/u-boot"
    UBOOT_BIN_PATH="${B}/${UBOOT_MACHINE_CLEAN}/u-boot.bin"
    UBOOT_DTB_PATH="${B}/${UBOOT_MACHINE_CLEAN}/dts/dt.dtb"
    DEFCONFIG_PATH="${B}/${UBOOT_MACHINE_CLEAN}/.config"

    if [ -e "${UBOOT_ELF_PATH}" ]; then
        install -Dm644 "${UBOOT_ELF_PATH}" "${DEPLOYDIR}/uboot-${MACHINE}-${PV}-${PR}.elf"
    else
        bbwarn "u-boot.elf not found at ${UBOOT_ELF_PATH}, skipping deploy"
    fi

    if [ -e "${UBOOT_BIN_PATH}" ]; then
        install -Dm644 "${UBOOT_BIN_PATH}" "${DEPLOYDIR}/uboot-${MACHINE}-${PV}-${PR}.bin"
    else
        bbwarn "u-boot.bin not found at ${UBOOT_BIN_PATH}, skipping deploy"
    fi

    if [ -e "${UBOOT_DTB_PATH}" ]; then
        install -Dm644 "${UBOOT_DTB_PATH}" "${DEPLOYDIR}/uboot-${MACHINE}-${PV}-${PR}.dtb"
    else
        bbwarn "dts/dt.dtb not found at ${UBOOT_DTB_PATH}, skipping deploy"
    fi

    install -Dm644 "${DEFCONFIG_PATH}" "${DEPLOYDIR}/uboot-defconfig-${MACHINE}-${PV}-${PR}.txt"
}

do_trim_header() {
    orig="${DEPLOYDIR}/${UBOOT_BINARY}"
    trimmed="${DEPLOYDIR}/${UBOOT_BINARY}.nohdr"

    if [ -f "$orig" ] && [[ "$orig" == *.imx ]]; then
        bbwarn "✂️ Trimming ${orig}, removing first 0x400 bytes → ${trimmed}"
        dd if="$orig" of="$trimmed" bs=1 skip=1024 status=none
    else
        bbwarn "Skipping trim: file not found or not a .imx → ${orig}"
    fi
}
do_deploy[postfuncs] += "do_trim_header"

EXTRA_OEMAKE:append = " KCFLAGS='-Oz -fdata-sections -ffunction-sections -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-ident -fno-stack-protector -fno-exceptions -fomit-frame-pointer' LDFLAGS='-Wl,--gc-sections -Wl,--icf=all -Wl,--no-undefined -Wl,--strip-all'"

# [hack] build debug version of uboot
# EXTRA_OEMAKE:append = " KCFLAGS='-O2 -g -fno-inline'"
# UBOOT_SUFFIX:forcevariable = "elf"
# DEBUG_BUILD:forcevariable = "1"
# UBOOT_MAKE_TARGET:forcevariable = "u-boot.elf"
# INHIBIT_PACKAGE_STRIP:forcevariable = "1"
# INHIBIT_PACKAGE_DEBUG_SPLIT:forcevariable = "1"
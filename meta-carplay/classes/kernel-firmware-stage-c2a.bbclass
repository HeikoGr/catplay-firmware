C2A_KERNEL_BUILD_FIRMWARE_STAGE ?= "linux-firmware"
DEPENDS += "${C2A_KERNEL_BUILD_FIRMWARE_STAGE}"

do_configure:append() {
    if [ -z "${C2A_KERNEL_BUILD_FIRMWARE_STAGE}" ]; then
        bbwarn "Skipping kernel build firmware staging"
        return
    fi

    bbwarn "Staging firmware for kernel build now: ${C2A_KERNEL_BUILD_FIRMWARE_STAGE}"
    mkdir -p ${S}/c2a-firmware-stage
    cp -r ${RECIPE_SYSROOT}/lib/firmware/* ${S}/c2a-firmware-stage/
}

do_kernel_configme:append() {
    echo 'CONFIG_EXTRA_FIRMWARE_DIR="c2a-firmware-stage"' >> ${B}/.config
}

do_configure[vardeps] += "C2A_KERNEL_BUILD_FIRMWARE_STAGE"

# Copy the module to sysroot for firmware bundling purposes
# without depending on runtime-generated kernel-module-xxx split-type pkg
DRIVER_DIR ?= "drivers/extra/"

do_install() {
    for file in $(ls ${S} | grep "\.ko"); do
        bbwarn "Installing ${file} kernel module";
        install -Dm 0644 ${S}/${file} ${D}${nonarch_base_libdir}/modules/${KERNEL_VERSION}/kernel/${DRIVER_DIR}/${file};
    done
}

FILES:${PN} = "/lib/modules/${KERNEL_VERSION}/kernel/${DRIVER_DIR}/*"

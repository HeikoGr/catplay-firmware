SUMMARY = "Allwinner Brandy boot binaries for sun300iw1p1/V821"
DESCRIPTION = "Builds vendor Brandy simpleboot/falcon and FES binaries for V821."
LICENSE = "CLOSED"

inherit deploy

SRC_URI = "git://github.com/sam-yangjj/tina-v821-v1.3-brandy.git;branch=main;protocol=https"
SRC_URI += "file://modern-linker.patch file://0001-sun300iw1p1-v821-falcon-fes-poc.patch file://0002-sun300iw1p1-v821-add-falcon2-rtc-flash-checkpoint.patch"
SRC_URI += "file://0003-sun300iw1p1-falcon-preserve-fel-dram.patch"
SRC_URI += "file://0004-sun300iw1p1-initialize-plls-in-simpleboot.patch"

ERROR_QA:remove = "patch-fuzz"
PACKAGE_ARCH = "${MACHINE_ARCH}"

PROVIDES = "virtual/bootloader"

PR = "r2"

SRCREV = "34809037526678ccda1720cb4b4ec7ee32272c38"
PV = "1.3git${SRCPV}"

S = "${UNPACKDIR}/${BP}/brandy-2.0/spl"
B = "${WORKDIR}/build"

BRANDY_PLATFORM = "sun300iw1p1"
BRANDY_IC = "v821"

ANDES_TOOLCHAIN_DIR = "${STAGING_LIBDIR_NATIVE}/sunxi-toolchains/nds32le-linux-glibc-v5d"
ANDES_CROSS_COMPILE = "${ANDES_TOOLCHAIN_DIR}/bin/riscv32-unknown-linux-"
ANDES_GCC_INCLUDE = "${ANDES_TOOLCHAIN_DIR}/lib/gcc/riscv32-linux/12.2.0/include"
ANDES_COMPILEINC = "-isystem ${ANDES_GCC_INCLUDE}"

XUANTIE_TOOLCHAIN_NAME = "Xuantie-900-gcc-linux-5.10.4-glibc-x86_64-V2.8.1"
XUANTIE_TOOLCHAIN_DIR = "${STAGING_LIBDIR_NATIVE}/sunxi-toolchains/${XUANTIE_TOOLCHAIN_NAME}"
XUANTIE_CROSS_COMPILE = "${XUANTIE_TOOLCHAIN_DIR}/bin/riscv64-unknown-linux-gnu-"
XUANTIE_GCC_INCLUDE = "${XUANTIE_TOOLCHAIN_DIR}/lib/gcc/riscv64-unknown-linux-gnu/10.4.0/include"
XUANTIE_COMPILEINC = "-isystem ${XUANTIE_GCC_INCLUDE}"

BRANDY_SIMPLEBOOT_BIN = "simpleboot_falcon_${BRANDY_PLATFORM}.bin"
BRANDY_SIMPLEBOOT2_BIN = "simpleboot_falcon2_${BRANDY_PLATFORM}.bin"
BRANDY_FES_BIN = "fes1_${BRANDY_PLATFORM}.bin"

# Brandy invokes raw vendor GCC/LD and appends Make/environment flags into
# ALL_CFLAGS/ALL_AFLAGS, so keep Yocto target flags out of this firmware build.
BRANDY_MAKE_ENV = "CFLAGS= CPPFLAGS= ASFLAGS= AFLAGS= LDFLAGS= HOSTCFLAGS= HOSTCPPFLAGS= HOSTLDFLAGS="

DEPENDS += "python3-native sunxi-nds32le-native sunxi-xuantie-native"


do_configure[cleandirs] = "${B}"
do_configure() {
    test -x "${ANDES_CROSS_COMPILE}gcc" || bbfatal "Missing Andes compiler: ${ANDES_CROSS_COMPILE}gcc"
    test -f "${ANDES_GCC_INCLUDE}/stddef.h" || bbfatal "Missing Andes GCC stddef.h: ${ANDES_GCC_INCLUDE}/stddef.h"
    test -x "${XUANTIE_CROSS_COMPILE}gcc" || bbfatal "Missing Xuantie compiler: ${XUANTIE_CROSS_COMPILE}gcc"
    test -f "${XUANTIE_GCC_INCLUDE}/stddef.h" || bbfatal "Missing Xuantie GCC stddef.h: ${XUANTIE_GCC_INCLUDE}/stddef.h"

    oe_runmake -C "${S}" ${BRANDY_MAKE_ENV} p=${BRANDY_PLATFORM}
}

brandy_debug_falcon_failure() {
    status=${1:-$?}
    echo "top-level falcon build failed with status ${status}"
    if [ -f "${S}/nboot/boot0_falcon.bin" ]; then
        ls -l "${S}/nboot/boot0_falcon.bin" || true
        echo "rerun gen_check_sum without stdout redirection:"
        (cd "${S}/nboot" && "${S}/mk/gen_check_sum" "${S}/nboot/boot0_falcon.bin" boot0_falcon_${BRANDY_PLATFORM}.bin) || true
        if [ -f "${S}/nboot/boot0_falcon_${BRANDY_PLATFORM}.bin" ]; then
            ls -l "${S}/nboot/boot0_falcon_${BRANDY_PLATFORM}.bin" || true
            echo "rerun encrypto_boot0 without stdout redirection:"
            (cd "${S}/nboot" && "${S}/mk/encrypto_boot0" -f boot0_falcon_${BRANDY_PLATFORM}.bin -c ${BRANDY_IC}) || true
        fi
    else
        echo "${S}/nboot/boot0_falcon.bin was not created"
    fi
    return ${status}
}

do_compile() {
    set +e
    make ${PARALLEL_MAKE} -C "${S}" \
        ${BRANDY_MAKE_ENV} \
        CROSS_COMPILE="${ANDES_CROSS_COMPILE}" \
        COMPILEINC="${ANDES_COMPILEINC}" \
        LICHEE_IC=${BRANDY_IC} \
        -B falcon
    falcon_status=$?
    set -e
    if [ ${falcon_status} -ne 0 ]; then
        brandy_debug_falcon_failure ${falcon_status}
    fi

    oe_runmake -C "${S}" \
        ${BRANDY_MAKE_ENV} \
        CROSS_COMPILE="${ANDES_CROSS_COMPILE}" \
        COMPILEINC="${ANDES_COMPILEINC}" \
        LICHEE_IC=${BRANDY_IC} \
        -B falcon2

    oe_runmake -C "${S}/simpleboot" \
        ${BRANDY_MAKE_ENV} \
        TOPDIR="${S}" SRCTREE="${S}" PLATFORM=${BRANDY_PLATFORM} \
        ARCH=riscv CPU=ads_rv32 CROSS_COMPILE="${ANDES_CROSS_COMPILE}" \
        COMPILEINC="${ANDES_COMPILEINC}" LICHEE_IC=${BRANDY_IC} \
        CMD_ECHO_SILENT=echo Q=@ CP=true MKDIR=true \
        -B falcon

    oe_runmake -C "${S}/simpleboot" \
        ${BRANDY_MAKE_ENV} \
        TOPDIR="${S}" SRCTREE="${S}" PLATFORM=${BRANDY_PLATFORM} \
        ARCH=riscv CPU=ads_rv32 CROSS_COMPILE="${ANDES_CROSS_COMPILE}" \
        COMPILEINC="${ANDES_COMPILEINC}" LICHEE_IC=${BRANDY_IC} \
        CMD_ECHO_SILENT=echo Q=@ CP=true MKDIR=true \
        -B falcon2

    oe_runmake -C "${S}/fes" \
        ${BRANDY_MAKE_ENV} \
        TOPDIR="${S}" SRCTREE="${S}" PLATFORM=${BRANDY_PLATFORM} \
        ARCH=riscv CPU=riscv32 CROSS_COMPILE="${XUANTIE_CROSS_COMPILE}" \
        COMPILEINC="${XUANTIE_COMPILEINC}" LICHEE_IC=${BRANDY_IC} \
        CMD_ECHO_SILENT=echo Q=@ CP=true MKDIR=true \
        -B fes

    test -f "${S}/simpleboot/${BRANDY_SIMPLEBOOT_BIN}" || bbfatal "Missing ${BRANDY_SIMPLEBOOT_BIN}"
    test -f "${S}/simpleboot/${BRANDY_SIMPLEBOOT2_BIN}" || bbfatal "Missing ${BRANDY_SIMPLEBOOT2_BIN}"
    test -f "${S}/fes/${BRANDY_FES_BIN}" || bbfatal "Missing ${BRANDY_FES_BIN}"
}

do_install() {
    install -d "${D}${nonarch_base_libdir}/firmware/sunxi-brandy"
    install -m 0644 "${S}/simpleboot/${BRANDY_SIMPLEBOOT_BIN}" \
        "${D}${nonarch_base_libdir}/firmware/sunxi-brandy/simpleboot-v821b-falcon.bin"
    install -m 0644 "${S}/simpleboot/${BRANDY_SIMPLEBOOT2_BIN}" \
        "${D}${nonarch_base_libdir}/firmware/sunxi-brandy/simpleboot-v821b-falcon2.bin"
    install -m 0644 "${S}/fes/${BRANDY_FES_BIN}" \
        "${D}${nonarch_base_libdir}/firmware/sunxi-brandy/fes1-sun300iw1p1-v821b.bin"
}

FILES:${PN} += "${nonarch_base_libdir}/firmware/sunxi-brandy"

PACKAGE_ARCH = "${MACHINE_ARCH}"

COMPATIBLE_MACHINE = "(woo|woo-recov)"

do_deploy() {
    install -d "${DEPLOYDIR}/sunxi-brandy"
    install -m 0644 "${S}/simpleboot/${BRANDY_SIMPLEBOOT_BIN}" \
        "${DEPLOYDIR}/sunxi-brandy/simpleboot-v821b-falcon.bin"
    install -m 0644 "${S}/simpleboot/${BRANDY_SIMPLEBOOT2_BIN}" \
        "${DEPLOYDIR}/sunxi-brandy/simpleboot-v821b-falcon2.bin"
    install -m 0644 "${S}/fes/${BRANDY_FES_BIN}" \
        "${DEPLOYDIR}/sunxi-brandy/fes1-sun300iw1p1-v821b.bin"
}

addtask do_deploy before do_build after do_compile

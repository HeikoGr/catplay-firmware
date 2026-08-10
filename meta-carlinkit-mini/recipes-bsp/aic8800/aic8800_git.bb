SUMMARY = "AIC8800 vendor driver"
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

SRCBRANCH = "main"
AIC_SRC = "git://github.com/radxa-pkg/aic8800;protocol=https"

SRC_URI = " \
    ${AIC_SRC};branch=${SRCBRANCH} \
    file://fix-linux-7.2-sdio-build.patch \
    file://regulators.patch \
    file://ax1800.patch \
    file://rwnx_send_bcn.patch \
    file://set_mon_chan.patch \
    file://mips-pc-macro-fix.patch \
    file://rwnx-msg-tx-add-vmalloc-include.patch \
    file://config.patch \
    file://d80-rf-calib-file.patch \
    file://rwnx_cfg80211_init.patch \
"

FILESEXTRAPATHS:prepend := "${THISDIR}/files:"

SRCREV = "6e076049b719ac2ff7ce5c92786a680407b11cdb"

COMPILE_DIR = "${WORKDIR}/git/src/SDIO/driver_fw/driver/aic8800"
FIRMWARE_DIR = "${WORKDIR}/git/src/SDIO/driver_fw/fw/"
SRC_DIR = "${WORKDIR}/git"

S = "${COMPILE_DIR}"
B = "${WORKDIR}/build"

PACKAGES =+ "\
    ${PN}-fw-aic8800 \
    ${PN}-fw-aic8800d80 \
    ${PN}-fw-aic8800d80x2 \
    ${PN}-fw-aic8800dc \
"

inherit module
do_patch[depends] += "dos2unix-native:do_populate_sysroot"

do_patch:prepend() {
    bb.build.exec_func('do_patch_aic', d)
}

do_patch_aic() {
    # Retries can run do_patch on a partially patched tree.
    # Reset to pristine SRCREV to make patching deterministic.
    if [ -d ${WORKDIR}/git/.git ] && command -v git >/dev/null 2>&1; then
        cd ${WORKDIR}/git
        git reset --hard ${SRCREV} >/dev/null 2>&1 || git reset --hard >/dev/null 2>&1 || true
        git clean -fd >/dev/null 2>&1 || true
    fi

    # Normalize full source tree line endings before debian patch series.
    find ${SRC_DIR} -type f -exec dos2unix {} \; || true

    for i in $(cat ${WORKDIR}/git/debian/patches/series); do 
        cd ${WORKDIR}/git/ && patch --batch -p1 < ${WORKDIR}/git/debian/patches/$i || (echo "Failed to apply patch $i" && exit 1); 
    done
}

do_install:append(){
    for i in $(ls ${FIRMWARE_DIR}); do
        # New upstream releases may ship archive files (e.g. *.7z)
        # next to firmware directories; only process directory entries.
        if [ ! -d ${FIRMWARE_DIR}/${i} ]; then
            continue
        fi
        for j in $(ls ${FIRMWARE_DIR}/${i}); do
            install -Dm 0755 ${FIRMWARE_DIR}/${i}/${j} ${D}/${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/${i}/${j}
        done
    done
}

EXTRA_OEMAKE += "KERNELDIR=${STAGING_KERNEL_BUILDDIR} -C ${STAGING_KERNEL_BUILDDIR} M=${S}"

FILES:${PN}-fw-aic8800 = "\
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800/* \
"
FILES:${PN}-fw-aic8800d80 = "\
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800D80/fw_patch_table_8800d80_u02.bin \
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800D80/fw_adid_8800d80_u02.bin \
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800D80/fw_patch_8800d80_u02.bin \
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800D80/fw_patch_8800d80_u02_ext0.bin \
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800D80/fmacfw_8800d80_h_u02.bin \
"
FILES:${PN}-fw-aic8800d80x2 = "\
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800D80X2/* \
"
FILES:${PN}-fw-aic8800dc = "\
    ${nonarch_base_libdir}/firmware/aic8800_fw/SDIO/aic8800DC/* \
"

# FILES:${PN} = ""
ALLOW_EMPTY:${PN} = "1"
INSANE_SKIP:${PN} += "installed-vs-shipped"

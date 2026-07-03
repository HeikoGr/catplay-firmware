LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/files/common-licenses/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

SRC_URI += " \
    file://aic_userconfig_8800d80.txt \
    file://aic_rf_calib.bin \
"

S = "${WORKDIR}"

inherit allarch

do_install() {
    for chip in ${S}/*; do
        [ -f "${chip}" ] || continue
        install -Dm 644 "${chip}" "${D}/lib/firmware/aic8800_fw/SDIO/aic8800D80/$(basename "${chip}")"
    done
}

FILES:${PN} += "/"

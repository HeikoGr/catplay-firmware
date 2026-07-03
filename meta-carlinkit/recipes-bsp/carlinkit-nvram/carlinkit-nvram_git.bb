LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/files/common-licenses/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

SRC_URI += "file://nvram_bcm4335.txt \
            file://nvram_bcm4354.txt \
            file://nvram_bcm4358.txt \
"

SRC_URI += "file://rtl8822b_config.bin \
            file://rtl8822cs_config.bin \
"

S = "${WORKDIR}"

inherit allarch

do_install() {
    for chip in $(echo 4335 4354 4358); do
        install -Dm 644 ${S}/nvram_bcm${chip}.txt ${D}/lib/firmware/brcm/brcmfmac${chip}-sdio.txt
    done

    install -Dm 644 ${S}/rtl8822b_config.bin ${D}/lib/firmware/rtl_bt/rtl8822b_config.bin
    install -Dm 644 ${S}/rtl8822cs_config.bin ${D}/lib/firmware/rtl_bt/rtl8822cs_config.bin
}

FILES:${PN} += "/"

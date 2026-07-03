SUMMARY = "Init script for C2A"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/files/common-licenses/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

SRC_URI += "file://carlinkit-init.sh"
SRC_URI += "file://init.sh"

S = "${WORKDIR}"

TARGET_DIR = "${nonarch_libdir}/${PN}/"

do_install() {
    install -Dm 0755 ${S}/carlinkit-init.sh ${D}/${sysconfdir}/init.d/carlinkit-init

    for i in $(echo init.sh); do
        install -Dm 0755 ${WORKDIR}/${i} ${D}/${TARGET_DIR}/${i};
    done
}

inherit update-rc.d

INITSCRIPT_NAME = "carlinkit-init"
INITSCRIPT_PARAMS = "defaults 01 S"

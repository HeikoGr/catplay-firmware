SUMMARY = "C2A - overlays"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/files/common-licenses/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"
PR = "r1"

S = "${WORKDIR}"

inherit update-rc.d

INITSCRIPT_NAME = "c2a-overlay-init"
INITSCRIPT_PARAMS = "start 03 S ."

FILESEXTRAPATHS:prepend := "${THISDIR}/files:"

SRC_URI += "file://c2a-overlay-init.sh file://c2a-overlay-init.c"

C2A_OVERLAY_INIT_FAST ?= "1"

S = "${WORKDIR}"

do_compile() {
    ${CC} ${CFLAGS} ${LDFLAGS} -static -Wall \
        ${WORKDIR}/c2a-overlay-init.c -o c2a-overlay-init || \
    ${CC} ${CFLAGS} ${LDFLAGS} -Wall \
        ${WORKDIR}/c2a-overlay-init.c -o c2a-overlay-init
}

do_install:append() {
	if [ "${C2A_OVERLAY_INIT_FAST}" = "1" ]; then
		install -Dm 0755 ${B}/c2a-overlay-init ${D}/${sysconfdir}/init.d/c2a-overlay-init
	else
		install -Dm 0755 ${S}/c2a-overlay-init.sh ${D}/${sysconfdir}/init.d/c2a-overlay-init
	fi
}
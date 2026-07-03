inherit update-rc.d

INITSCRIPT_NAME = "catplay"
INITSCRIPT_PARAMS = "disable"

FILESEXTRAPATHS:prepend := "${THISDIR}/files:"

SRC_URI += "file://catplay.sh"

do_install:append() {
	install -Dm 0755 ${S}/catplay.sh ${D}/${sysconfdir}/init.d/catplay
}
FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"
BUSYBOX_SPLIT_SUID = "0"

SRC_URI += "file://busybox-more.cfg file://inittab"


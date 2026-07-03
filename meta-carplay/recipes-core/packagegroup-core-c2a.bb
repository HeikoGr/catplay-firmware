SUMMARY = "Minimal base package group"
LICENSE = "MIT"
PR = "r1"

PACKAGE_ARCH = "${MACHINE_ARCH}"

inherit packagegroup

RDEPENDS:${PN} = "${C2A_SYSTEM_PACKAGES}"

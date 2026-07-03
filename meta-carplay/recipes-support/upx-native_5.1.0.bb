SUMMARY = "Ultimate Packer for eXecutables (binary-only)"
DESCRIPTION = "UPX is a free, portable, extendable, high-performance executable packer"
HOMEPAGE = "https://upx.github.io/"
LICENSE = "GPL-2.0-or-later"
LIC_FILES_CHKSUM = "file://LICENSE;md5=353753597aa110e0ded3508408c6374a"

SRC_URI = "https://github.com/upx/upx/releases/download/v${PV}/upx-${PV}-amd64_linux.tar.xz"
SRC_URI[sha256sum] = "946b7269d0f7fcc1c5da0f771ea7fe9c0fed3534cf41d285980830019b4bc95e"

S = "${WORKDIR}/upx-${PV}-amd64_linux"

inherit native

do_install() {
    install -Dm 0755 ${S}/upx ${D}${bindir}/upx
}

COMPATIBLE_HOST = "x86_64.*-linux"
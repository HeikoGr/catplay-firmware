C2A_DROPBEAR_KEY_PREGEN ?= "1"
PACKAGE_ARCH = "${MACHINE_ARCH}"

FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"

SRC_URI += "file://dropbear_rsa_host_key"
SRC_URI += "file://dropbear_rsa_host_key.pub"

PR = "r2"

do_install:append() {
    if [ "x${C2A_DROPBEAR_KEY_PREGEN}" = "x1" ]; then
        bbwarn "Adding ssh key pregen"
        install -Dm 0600 ${WORKDIR}/dropbear_rsa_host_key ${D}${sysconfdir}/dropbear/dropbear_rsa_host_key
    else
        bbwarn "Not adding ssh key pregen"
    fi
}

# do_compile[depends] += "openssh-keygen:do_populate_sysroot"
do_compile[vardeps] += "C2A_DROPBEAR_KEY_PREGEN"
do_install[vardeps] += "C2A_DROPBEAR_KEY_PREGEN"

# do_generate_host_key() {
#     ssh-keygen -t rsa -m PEM -f ${WORKDIR}/dropbear_rsa_host_key -N ""
# }

# addtask generate_host_key before do_configure after do_unpack

inherit image

SUMMARY = "Minimal image with hello.txt"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COREBASE}/meta/files/common-licenses/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

IMAGE_INSTALL = ""
IMAGE_LINGUAS = ""
EXTRA_IMAGE_FEATURES = ""
IMAGE_FEATURES = ""
PACKAGE_INSTALL = ""
GLIBC_GENERATE_LOCALES = ""
USE_NLS = "no"
INHIBIT_DEFAULT_DEPS = "1"

ROOTFS_POSTPROCESS_COMMAND += "add_hello_txt;"

python add_hello_txt() {
    import os
    rootfs = d.getVar("IMAGE_ROOTFS")
    path = os.path.join(rootfs, "hello.txt")
    with open(path, "w") as f:
        f.write("Hello from Yocto!!!\n")
}

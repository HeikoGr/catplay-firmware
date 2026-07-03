DEPENDS:remove = "glib-2.0"

PACKAGECONFIG = ""

EXTRA_OECONF:append = " \
    --disable-tests \
    --disable-asserts \
    --disable-verbose-mode \
    --disable-xml-docs \
    --disable-selinux \
    --disable-libaudit \
    --without-x \
    --without-systemdsystemunitdir \
    --disable-static \
"

do_install:append() {
    install -d ${D}${localstatedir}/lib/dbus
    printf "6199d9c6103b6e0629d80d4500000000" > ${D}${localstatedir}/lib/dbus/machine-id
}

FILES:${PN}:append = " ${localstatedir}/lib/dbus/machine-id"

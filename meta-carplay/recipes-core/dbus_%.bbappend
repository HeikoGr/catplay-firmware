DEPENDS:remove = "glib-2.0"

# D-Bus 1.16 no longer builds the daemon unless message-bus is selected.
# Keep the C2A build minimal (no user session, systemd, X11 or tests), while
# retaining the system bus and the service activation supported before the
# Meson-based recipe made these features explicit.
PACKAGECONFIG = "message-bus traditional-activation"

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

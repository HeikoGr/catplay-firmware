# Remove udev
PACKAGECONFIG:forcevariable = ""

EXTRA_OECONF:append = " --disable-shared --enable-static"
EXTRA_OECONF:remove = "--enable-shared"

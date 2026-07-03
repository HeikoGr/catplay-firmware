PACKAGECONFIG = ""
EXTRA_OEMESON:append = " \
  -Dxattr=false \
  -Dlibelf=disabled \
  -Ddtrace=false \
  -Dsystemtap=false \
  -Dman=false \
  -Dgtk_doc=false \
  -Dselinux=disabled \
  -Dnls=disabled \
  -Dtests=false \
  -Dglib_debug=disabled \
"
RRECOMMENDS:${PN}:remove = "shared-mime-info"

GTKDOC_ENABLED = "False"
DEPENDS:remove = "gtk-doc-native"
#DEPENDS:remove = "libpcre2"

# -Ddefault_library=both
EXTRA_OEMESON:append = " -Ddefault_library=static"
FILES:${PN}-dev += "${libdir}/*.a"

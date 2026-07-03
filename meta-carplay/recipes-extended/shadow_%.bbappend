# #FILES:${PN}-base:remove = "${base_bindir}/login.shadow"
# FILES:${PN}-base:remove = "${base_bindir}/su.shadow"
# FILES:${PN}-base:remove = "${bindir}/sg"
# FILES:${PN}-base:remove = "${bindir}/newgrp.shadow"
# FILES:${PN}-base:remove = "${bindir}/groups.shadow"

RDEPENDS:${PN}:remove = "util-linux-sulogin"

# # ALTERNATIVE:${PN} = ""
# # ALTERNATIVE:${PN}-base = ""

# # pkg_postinst:${PN}:class-target () {
# #     :
# # }
ALTERNATIVE_PRIORITY = "0"
ALTERNATIVE_PRIORITY[login] = "0"

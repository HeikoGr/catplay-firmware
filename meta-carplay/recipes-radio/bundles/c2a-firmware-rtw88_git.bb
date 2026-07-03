
C2A_FIRMWARE_STAGE = "linux-firmware-rtl8822"

PACKAGES =+ "${PN} ${PN}-rtl8822bs ${PN}-rtl8822cs"
RDEPENDS:${PN} += "${PN}-rtl8822bs ${PN}-rtl8822cs"
RDEPENDS:${PN}-rtl8822cs += "${PN}"
RDEPENDS:${PN}-rtl8822bs += "${PN}"

FILES:${PN} = " \
  /lib/modules/*/kernel/drivers/net/wireless/realtek/rtw88/rtw88_core.ko \
  /lib/modules/*/kernel/drivers/net/wireless/realtek/rtw88/rtw88_sdio.ko \
"

FILES:${PN}-rtl8822bs = " \
  /lib/firmware/rtl_bt/rtl8822b_fw.bin \
  /lib/firmware/rtw88/rtw8822b_fw.bin \
  \
  /lib/modules/*/kernel/drivers/net/wireless/realtek/rtw88/rtw88_8822b.ko \
  /lib/modules/*/kernel/drivers/net/wireless/realtek/rtw88/rtw88_8822bs.ko \
"

FILES:${PN}-rtl8822cs = " \
  /lib/firmware/rtl_bt/rtl8822cs_fw.bin \
  /lib/firmware/rtw88/rtw8822c_fw.bin \
  /lib/firmware/rtw88/rtw8822c_wow_fw.bin \
  \
  /lib/modules/*/kernel/drivers/net/wireless/realtek/rtw88/rtw88_8822c.ko \
  /lib/modules/*/kernel/drivers/net/wireless/realtek/rtw88/rtw88_8822cs.ko \
"
require staging-linux-firmware.inc
require staging-kernel-modules.inc
require bundle.inc
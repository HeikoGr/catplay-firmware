
C2A_FIRMWARE_STAGE = "linux-firmware"
C2A_MODULES_STAGE = "rtl8822cs"
DRIVER_PATH = "drivers/extra/88x2cs.ko"

FILES:${PN} = "/lib/modules/*/kernel/${DRIVER_PATH} /lib/firmware/rtl_bt/rtl8822cs_fw.bin"

require bundle.inc
require staging-kernel-modules.inc
require staging-linux-firmware.inc

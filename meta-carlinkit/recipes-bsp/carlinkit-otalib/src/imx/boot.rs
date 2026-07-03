use crate::{SystemUtil, dmesg, modprobe_util::ModprobeUtil};

pub struct ImxBootUtil {}

impl ImxBootUtil {
    // It's not clear what exactly GPIO6 does (original comment is "set quickly charge mode").
    // It's likely controlling a "charger id" chip either at the USB-C end or the USB-A end.
    pub fn setup_charger() {
        let _ = SystemUtil::run_shell(
            "echo 6 > /sys/class/gpio/export; echo out > /sys/class/gpio/gpio6/direction; echo 0 > /sys/class/gpio/gpio6/value",
        );
        let _ = SystemUtil::run_shell("sleep 0.1; echo 1 >/sys/class/gpio/gpio6/value");
    }

    /// Start RGB driver.
    pub fn setup_rgb() {
        dmesg!("[boot] Starting RGB");
        let _ = ModprobeUtil::modprobe("carlinkit_rgb");
        let _ = SystemUtil::run_shell("echo 255 > /sys/class/leds/rgb/brightness");
        let _ = SystemUtil::run_shell("echo 255 0 0 > /sys/class/leds/rgb/multi_intensity");
    }

    /// Setup "extra UDC" (the USB-A port) for mass storage/charging and main UDC for CatPlay.
    pub fn setup_usb() {
        dmesg!("[boot] Starting charger id");
        let _ = ModprobeUtil::modprobe("ci_hdrc_imx");
        let _ = SystemUtil::write_file("/sys/class/usb_role/ci_hdrc.0-role-switch/role", "host");
        let _ = SystemUtil::write_file("/sys/class/usb_role/ci_hdrc.1-role-switch/role", "device");
    }
}

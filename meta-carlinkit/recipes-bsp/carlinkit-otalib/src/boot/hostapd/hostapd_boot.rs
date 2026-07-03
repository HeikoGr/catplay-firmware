use core::time::Duration;

use crate::{SystemUtil, boot::hostapd::HostapdConfigParams, dmesg, nostd::SmallFd};

pub struct Hostapd(());

impl Hostapd {
    pub fn start(template: &str, params: &HostapdConfigParams<'_>) -> Result<(), &'static str> {
        let _ = SystemUtil::unlink_if_exists("/etc/hostapd.conf");

        let cfg = params.format(template).map_err(|_| "failed to format hostapd config")?;
        {
            let fd = SmallFd::create("/etc/hostapd.conf").map_err(|_| "failed to create file")?;
            let _ = fd.write(cfg.as_bytes());
        }

        let mut ret = Ok(());
        for i in 0..5 {
            dmesg!("[boot] Starting hostapd now (attempt {i})");
            ret = SystemUtil::run_shell("/etc/init.d/hostapd start");
            if ret.is_err() {
                dmesg!("[boot] hostapd returned non-zero exit-code!");
                SystemUtil::sleep(Duration::from_millis(500));
            } else {
                dmesg!("[boot] hostapd has started!");
                return Ok(());
            }
        }

        ret
    }

    pub fn start_p2p(template: &str, params: &HostapdConfigParams) -> Result<(), &'static str> {
        // only valid for aic8800 (p2p0)
        let _ = SystemUtil::unlink_if_exists("/etc/wpa_supplicant.conf");
        let cfg = params.format(template).map_err(|_| "failed to format wpa_supplicant config")?;

        let _ = SystemUtil::write_file("/etc/wpa_supplicant.conf", cfg.as_str());
        dmesg!("[boot] Starting wpa_supplicant");
        if SystemUtil::exec(
            "/usr/sbin/wpa_supplicant",
            &["-B", "-Dnl80211", "-ip2p0", "-c/etc/wpa_supplicant.conf"],
        )
        .is_err()
        {
            return Err("failed to start wpa_supplicant");
        }

        let mut pong = false;

        for i in 0..100 {
            if SystemUtil::exec("/usr/sbin/wpa_cli", &["ping"]).is_ok() {
                pong = true;
                dmesg!("[boot] wpa_cli pong OK at {i}");
                break;
            }
            SystemUtil::sleep(Duration::from_millis(20));
        }

        if !pong {
            return Err("wpa_cli does not respond to ping");
        }

        if SystemUtil::exec("/usr/sbin/wpa_cli", &["p2p_group_add", "persistent=0"]).is_err() {
            return Err("wpa_cli failed p2p_group_add persistent=0");
        }

        dmesg!("[boot] wpa_supplicant now forked to background");
        Ok(())
    }
}

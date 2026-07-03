use core::fmt::Write;
use heapless::String;

pub struct HostapdConfigParams<'a> {
    pub ssid: &'a str,
    pub wpa_passphrase: &'a str,
    pub channel_override: Option<&'a str>,
}

impl<'a> HostapdConfigParams<'a> {
    pub fn format(&self, template: &str) -> Result<String<2048>, &'static str> {
        const ERR_TOO_LONG: &str = "hostapd config too long";
        let mut rendered = String::new();
        macro_rules! pushf {
        ($($arg:tt)*) => {
            rendered
                .write_fmt(format_args!($($arg)*))
                .map_err(|_| ERR_TOO_LONG)?
        };
    }

        for line_with_nl in template.split_inclusive('\n') {
            let (line, has_nl) = if let Some(line) = line_with_nl.strip_suffix('\n') {
                (line, true)
            } else {
                (line_with_nl, false)
            };

            let trimmed = line.trim_start();
            let indent = &line[..line.len() - trimmed.len()];

            if line.starts_with("ssid=") {
                pushf!("ssid={}", self.ssid);
            } else if trimmed.starts_with("ssid=\"") {
                pushf!("{indent}ssid=\"{}\"", self.ssid);
            } else if line.starts_with("wpa_passphrase=") {
                pushf!("wpa_passphrase={}", self.wpa_passphrase);
            } else if trimmed.starts_with("psk=\"") {
                pushf!("{indent}psk=\"{}\"", self.wpa_passphrase);
            } else if line.starts_with("channel=") {
                if let Some(channel) = self.channel_override {
                    pushf!("channel={channel}");
                } else {
                    pushf!("{line}");
                }
            } else {
                pushf!("{line}");
            }

            if has_nl {
                pushf!("\n");
            }
        }

        Ok(rendered)
    }
}
#[test]
fn format_hostapd_p2p() {
    let template = r#"ctrl_interface=/var/run/wpa_supplicant
network={
    ssid="C2A_P2P"
    psk="1234qwer"
    key_mgmt=WPA-PSK
}
"#;
    let params = HostapdConfigParams {
        ssid: "ssid",
        wpa_passphrase: "pass",
        channel_override: None,
    };
    let config = params.format(template).unwrap();
    assert_eq!(
        config,
        r#"ctrl_interface=/var/run/wpa_supplicant
network={
    ssid="ssid"
    psk="pass"
    key_mgmt=WPA-PSK
}
"#
    );
}

#[test]
fn format_hostapd() {
    use crate::boot::hostapd::HOSTAPD_ULTRA;
    let params = HostapdConfigParams {
        ssid: "ssid",
        wpa_passphrase: "pass",
        channel_override: Some("44"),
    };
    let config = params.format(HOSTAPD_ULTRA).unwrap();

    assert_eq!(
        config,
        r#"interface=wlan0
driver=nl80211
channel=44
ieee80211n=1
ieee80211ac=1
ieee80211ax=1
vht_oper_chwidth=0
he_oper_chwidth=0
hw_mode=a
ignore_broadcast_ssid=0
wowlan_triggers=any
rsn_pairwise=CCMP
ssid=ssid
wpa=2
wpa_passphrase=pass
ap_max_inactivity=10
he_basic_mcs_nss_set=65532
vendor_elements=DD4500A0400000020022010A424D57436172506C61790203424D57030F4632352D4E425445766F2D3037313604030001A90606E03501B89DFC0706E03501B89DFCDD0400A04000

assocresp_elements=7f0400000080
"#
    );
}

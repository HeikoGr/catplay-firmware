use core::fmt::Write;
use heapless::String;

pub struct CatplayConfig<'a> {
    pub ssid: &'a str,
    pub wpa_passphrase: &'a str,
    pub channel: Option<&'a str>,

    pub wlan_dev: &'a str,
    pub hci_dev: &'a str,
    pub name: &'a str,
    pub udc: &'a str,
    pub udc_extra: Option<&'a str>,
    pub mfi_bus: &'a str,
    pub mfi_dev_addr: &'a str,
    pub disable_gadget: bool,
}

impl<'a> CatplayConfig<'a> {
    pub fn format(&self) -> Result<String<1024>, &'static str> {
        const ERR_TOO_LONG: &str = "catplay config too long";
        let mut rendered = String::new();
        macro_rules! pushf {
        ($($arg:tt)*) => {
            rendered
                .write_fmt(format_args!($($arg)*))
                .map_err(|_| ERR_TOO_LONG)?
        };
    }

        pushf!("debug = true\n");
        pushf!("persist_dir = \"/persist/\"\n\n");

        pushf!("[mfi]\n");
        pushf!("selftest = false\n\n");

        pushf!("[mfi.i2c]\n");
        pushf!("bus_offset = {}\n", self.mfi_bus);
        pushf!("dev_addr = {}\n", self.mfi_dev_addr);
        pushf!("timeout_ms = 1000\n\n");

        pushf!("[mfi.server]\n");
        pushf!("enabled = true\n");
        pushf!("bind = \"0.0.0.0:9000\"\n\n");

        pushf!("[bluetooth]\n");
        pushf!("enabled = false\n");
        pushf!("device = \"{}\"\n", self.hci_dev);
        pushf!("name = \"{}\"\n\n", self.name);

        pushf!("[wifi]\n");
        pushf!("enabled = false\n");
        pushf!("device = \"{}\"\n\n", self.wlan_dev);

        pushf!("[wifi_network]\n");
        pushf!("ssid = \"{}\"\n", self.ssid);
        pushf!("password = \"{}\"\n", self.wpa_passphrase);
        pushf!("wpa = true\n");
        if let Some(channel) = self.channel {
            pushf!("channel = {channel}\n");
        }
        pushf!("\n");

        pushf!("[gadget]\n");
        pushf!("enabled = {}\n", if self.disable_gadget { "false" } else { "true" });
        pushf!("pinned = false\n");
        if let Some(udc_extra) = self.udc_extra {
            pushf!("udc_extra = \"{udc_extra}\"\n");
        }
        pushf!("udc_car = \"{}\"\n", self.udc);

        Ok(rendered)
    }
}

#[test]
fn format_catplay() {
    let config = CatplayConfig {
        ssid: "ssid",
        wpa_passphrase: "pass",
        wlan_dev: "wlan0",
        hci_dev: "hci0",
        name: "name",
        udc: "udc",
        udc_extra: None,
        mfi_bus: "0",
        mfi_dev_addr: "0x10",
        disable_gadget: false,
        channel: Some("48"),
    }
    .format()
    .unwrap();
    assert_eq!(
        config,
        r#"debug = true
persist_dir = "/persist/"

[mfi]
selftest = false

[mfi.i2c]
bus_offset = 0
dev_addr = 0x10
timeout_ms = 1000

[mfi.server]
enabled = true
bind = "0.0.0.0:9000"

[bluetooth]
enabled = false
device = "hci0"
name = "name"

[wifi]
enabled = false
device = "wlan0"

[wifi_network]
ssid = "ssid"
password = "pass"
wpa = true
channel = 48

[gadget]
enabled = true
pinned = false
udc_car = "udc"
"#
    );
}

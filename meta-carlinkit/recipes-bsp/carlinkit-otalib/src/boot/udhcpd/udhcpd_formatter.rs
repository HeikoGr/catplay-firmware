use core::fmt::Write;
use heapless::String;

pub struct UdhcpdConfigParams<'a> {
    pub interface: &'a str,
    pub start: &'a str,
    pub end: &'a str,
    pub subnet: &'a str,
    pub instance: &'a str,
}

pub fn format_udhcpd_config(params: &UdhcpdConfigParams<'_>) -> Result<String<256>, &'static str> {
    const ERR_TOO_LONG: &str = "udhcpd config too long";
    let mut rendered = String::new();
    macro_rules! pushf {
        ($($arg:tt)*) => {
            rendered
                .write_fmt(format_args!($($arg)*))
                .map_err(|_| ERR_TOO_LONG)?
        };
    }

    pushf!("start\t\t{}\n", params.start);
    pushf!("end\t\t{}\n", params.end);
    pushf!("interface\t{}\n", params.interface);
    pushf!("lease_file\t/var/lib/udhcpd.{}.leases\n", params.instance);
    pushf!("option\tsubnet\t{}\n", params.subnet);
    pushf!("option\tlease\t864000 # 10 days\n");

    Ok(rendered)
}

#[test]
fn format_udhcpd() {
    let config = format_udhcpd_config(&UdhcpdConfigParams {
        interface: "wlan0",
        start: "192.168.50.100",
        end: "192.168.50.200",
        subnet: "255.255.0.0",
        instance: "wifi",
    })
    .unwrap();

    assert_eq!(
        config,
        "start\t\t192.168.50.100\n\
end\t\t192.168.50.200\n\
interface\twlan0\n\
lease_file\t/var/lib/udhcpd.wifi.leases\n\
option\tsubnet\t255.255.0.0\n\
option\tlease\t864000 # 10 days\n"
    );
}

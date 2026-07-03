use heapless::String;

use crate::SystemUtil;

pub fn apply_sysctl(values: &[(&str, &str)]) -> Result<(), &'static str> {
    for (key, value) in values {
        let mut path = String::<128>::new();
        path.push_str("/proc/sys/").map_err(|_| "sysctl path too long")?;
        for ch in key.chars() {
            if ch == '.' {
                path.push('/').map_err(|_| "sysctl path too long")?;
            } else {
                path.push(ch).map_err(|_| "sysctl path too long")?;
            }
        }

        SystemUtil::write_file(path.as_str(), value)?;
    }

    Ok(())
}

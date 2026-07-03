use core::slice;

use heapless::{String, Vec};

#[allow(clippy::missing_safety_doc)]
pub unsafe fn argv_to_heapless<const MAX_ARGS: usize, const MAX_LEN: usize>(
    argc: i32,
    argv: *const *const u8,
) -> Vec<String<MAX_LEN>, MAX_ARGS> {
    let mut out: Vec<String<MAX_LEN>, MAX_ARGS> = Vec::new();

    for i in 0..argc {
        unsafe {
            let cstr_ptr = *argv.add(i as usize);
            if cstr_ptr.is_null() {
                continue;
            }

            let mut len = 0usize;
            while *cstr_ptr.add(len) != 0 {
                len += 1;
            }

            let slice = slice::from_raw_parts(cstr_ptr, len);
            if let Ok(s) = str::from_utf8(slice) {
                let mut hs: String<MAX_LEN> = String::new();
                let _ = hs.push_str(s); // cut if too long
                let _ = out.push(hs);
            }
        }
    }

    out
}

#[macro_export]
macro_rules! stdout {
    // Without semicolon as separator to disambiguate between arms, Rust just
    // chooses the first so that the format string would land in $max.
    ($max:expr; $lenT:path; $($arg:tt)*) => {{
        let res = heapless::_export::format::<$max, $lenT>(core::format_args!($($arg)*));
        let res = res.unwrap();
        unsafe { libc::write(1, res.as_ptr() as *const _, res.len()) };
    }};
    ($max:expr; $($arg:tt)*) => {{
        let res = heapless::_export::format::<$max, usize>(core::format_args!($($arg)*));
        let res = res.unwrap();
        unsafe { libc::write(1, res.as_ptr() as *const _, res.len()) };
    }};
    ($($arg:tt)*) => {{
        let res = heapless::_export::format::<4096, usize>(core::format_args!($($arg)*));
        let res = res.unwrap();
        unsafe { libc::write(1, res.as_ptr() as *const _, res.len()) };
    }};
}

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => {
        $crate::stdout!($($t)*);
        $crate::stdout!("\n");
    };
}

#[macro_export]
macro_rules! dmesg {
    // Without semicolon as separator to disambiguate between arms, Rust just
    // chooses the first so that the format string would land in $max.
    ($max:expr; $lenT:path; $($arg:tt)*) => {{
        let res = heapless::_export::format::<$max, $lenT>(core::format_args!($($arg)*));
        let res = res.unwrap();
        if let Ok(fd) = $crate::nostd::SmallFd::open("/dev/kmsg") {
            let _ = fd.write(res.as_bytes());
        }
    }};
    ($max:expr; $($arg:tt)*) => {{
        let res = heapless::_export::format::<$max, usize>(core::format_args!($($arg)*));
        let res = res.unwrap();
        if let Ok(fd) = $crate::nostd::SmallFd::open("/dev/kmsg") {
            let _ = fd.write(res.as_bytes());
        }
    }};
    ($($arg:tt)*) => {{
        let res = heapless::_export::format::<4096, usize>(core::format_args!($($arg)*));
        let res = res.unwrap();
        if let Ok(fd) = $crate::nostd::SmallFd::open("/dev/kmsg") {
            let _ = fd.write(res.as_bytes());
        }
    }};
}

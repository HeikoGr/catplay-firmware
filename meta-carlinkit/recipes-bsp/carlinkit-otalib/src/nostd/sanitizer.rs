use core::iter::Peekable;
use core::str::Chars;
use heapless::String;

pub fn sanitize_filename<const N: usize>(input: &str) -> String<N> {
    let mut out = String::<N>::new();
    let mut iter: Peekable<Chars> = input.chars().peekable();

    while let Some(c) = iter.next() {
        match c {
            '/' | '\\' => continue,
            '.' => {
                if let Some('.') = iter.peek() {
                    iter.next();
                    continue;
                } else {
                    let _ = out.push('.');
                }
            }
            _ => {
                let _ = out.push(c);
            }
        }
    }

    if out.is_empty() {
        let _ = out.push_str("unnamed");
    }

    out
}

#[test]
fn test_sanitize_filename() {
    assert_eq!(sanitize_filename::<1024>("../rootfs.squashfs"), "rootfs.squashfs");
    assert_eq!(sanitize_filename::<1024>("rootfs.squashfs"), "rootfs.squashfs");
}

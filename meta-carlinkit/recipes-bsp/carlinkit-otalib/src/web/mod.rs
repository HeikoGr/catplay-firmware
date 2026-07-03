extern crate alloc;
extern crate libc;

use alloc::format;
use core::mem::{MaybeUninit, size_of};

mod props;

use props::{PropStore, props_to_json};

#[derive(Copy, Clone, Debug)]
pub struct Errno(pub i32);

struct SocketFd {
    fd: libc::c_int,
}

impl SocketFd {
    fn new(fd: libc::c_int) -> Self {
        Self { fd }
    }
}

impl Drop for SocketFd {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

pub struct WebServer {
    listen_fd: SocketFd,
    props: PropStore,
}

struct HttpRequest<'a> {
    method: &'a str,
    path: &'a str,
    body: &'a [u8],
}

impl WebServer {
    pub fn new(port: u16) -> Result<Self, Errno> {
        unsafe {
            let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
            if fd < 0 {
                return Err(Errno(*libc::__errno_location()));
            }

            let yes: libc::c_int = 1;
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &yes as *const _ as *const libc::c_void,
                size_of::<libc::c_int>() as _,
            );

            let mut addr: libc::sockaddr_in = core::mem::zeroed();
            addr.sin_family = libc::AF_INET as _;
            addr.sin_port = port.to_be();
            addr.sin_addr = libc::in_addr {
                s_addr: libc::INADDR_ANY.to_be(),
            };

            if libc::bind(fd, &addr as *const _ as *const libc::sockaddr, size_of::<libc::sockaddr_in>() as _) < 0 {
                let e = *libc::__errno_location();
                libc::close(fd);
                return Err(Errno(e));
            }

            if libc::listen(fd, 16) < 0 {
                let e = *libc::__errno_location();
                libc::close(fd);
                return Err(Errno(e));
            }

            Ok(Self {
                listen_fd: SocketFd::new(fd),
                props: PropStore::load(),
            })
        }
    }

    pub fn run_forked(mut self) -> Result<(), Errno> {
        unsafe {
            let pid = libc::fork();
            if pid < 0 {
                return Err(Errno(*libc::__errno_location()));
            }

            if pid > 0 {
                return Ok(());
            }

            if libc::setsid() < 0 {
                libc::_exit(1);
            }

            self.run()
        }
    }

    fn run(&mut self) -> ! {
        loop {
            let mut peer = MaybeUninit::<libc::sockaddr_in>::uninit();
            let mut len = size_of::<libc::sockaddr_in>() as libc::socklen_t;

            unsafe {
                let cfd = libc::accept(self.listen_fd.fd, peer.as_mut_ptr() as *mut _, &mut len);
                if cfd < 0 {
                    continue;
                }

                self.handle_client(cfd);
                libc::close(cfd);
            }
        }
    }

    fn handle_client(&mut self, cfd: libc::c_int) {
        let mut req_buf = [0u8; 8192];
        let req_len = match read_request(cfd, &mut req_buf) {
            Some(v) => v,
            None => {
                let _ = write_response(cfd, "400 Bad Request", "text/plain; charset=utf-8", b"bad request");
                return;
            }
        };

        let req = match parse_http_request(&req_buf[..req_len]) {
            Some(v) => v,
            None => {
                let _ = write_response(cfd, "400 Bad Request", "text/plain; charset=utf-8", b"bad request");
                return;
            }
        };

        if req.method == "GET" && req.path == "/" {
            let _ = write_response(cfd, "200 OK", "text/html; charset=utf-8", GUI_HTML.as_bytes());
            return;
        }

        if req.method == "GET" && req.path == "/props" {
            let body = props_to_json(self.props.all());
            let _ = write_response(cfd, "200 OK", "application/json; charset=utf-8", body.as_bytes());
            return;
        }

        if let Some(name) = req.path.strip_prefix("/prop/") {
            if req.method == "GET" {
                match self.props.get(name) {
                    Some(value) => {
                        let _ = write_response(cfd, "200 OK", "text/plain; charset=utf-8", value.as_bytes());
                    }
                    None => {
                        let _ = write_response(cfd, "404 Not Found", "text/plain; charset=utf-8", b"unknown prop");
                    }
                }
                return;
            }

            if req.method == "PUT" {
                let value = match core::str::from_utf8(req.body) {
                    Ok(v) => v.trim_end_matches(['\r', '\n']),
                    Err(_) => {
                        let _ = write_response(cfd, "400 Bad Request", "text/plain; charset=utf-8", b"value is not utf8");
                        return;
                    }
                };

                match self.props.set(name, value) {
                    Ok(new_value) => {
                        let _ = write_response(cfd, "200 OK", "text/plain; charset=utf-8", new_value.as_bytes());
                    }
                    Err(msg) => {
                        let _ = write_response(cfd, "400 Bad Request", "text/plain; charset=utf-8", msg.as_bytes());
                    }
                }
                return;
            }
        }

        let _ = write_response(cfd, "404 Not Found", "text/plain; charset=utf-8", b"not found");
    }
}

fn read_request(fd: libc::c_int, out: &mut [u8]) -> Option<usize> {
    let mut off = 0usize;
    let mut expected_total = None;

    while off < out.len() {
        let n = unsafe { libc::read(fd, out[off..].as_mut_ptr() as *mut _, out.len() - off) };
        if n <= 0 {
            return None;
        }
        off += n as usize;

        if expected_total.is_none() {
            if let Some(h_end) = find_header_terminator(&out[..off]) {
                let body_len = parse_content_length(&out[..h_end]).unwrap_or(0);
                expected_total = Some(h_end + 4 + body_len);
            }
        }

        if let Some(total) = expected_total {
            if off >= total {
                return Some(total);
            }
        }
    }

    None
}

fn parse_http_request(buf: &[u8]) -> Option<HttpRequest<'_>> {
    let header_end = find_header_terminator(buf)?;
    let line_end = find_crlf(buf)?;

    let line = core::str::from_utf8(&buf[..line_end]).ok()?;
    let mut parts = line.splitn(3, ' ');
    let method = parts.next()?;
    let path = parts.next()?;
    let version = parts.next()?;

    if !version.starts_with("HTTP/1.") {
        return None;
    }
    if !path.starts_with('/') {
        return None;
    }

    let body = &buf[header_end + 4..];
    Some(HttpRequest { method, path, body })
}

fn find_header_terminator(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }

    let mut i = 0usize;
    while i + 4 <= buf.len() {
        if &buf[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn parse_content_length(header: &[u8]) -> Option<usize> {
    let header = core::str::from_utf8(header).ok()?;
    for line in header.lines() {
        let line = line.trim();
        if line.len() >= 15 && line[..15].eq_ignore_ascii_case("content-length:") {
            let value = line[15..].trim();
            return value.parse::<usize>().ok();
        }
    }
    None
}

fn find_crlf(buf: &[u8]) -> Option<usize> {
    let mut i = 0usize;
    while i + 2 <= buf.len() {
        if &buf[i..i + 2] == b"\r\n" {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn write_all(fd: libc::c_int, data: &[u8]) -> Result<(), Errno> {
    let mut off = 0usize;
    while off < data.len() {
        let n = unsafe { libc::write(fd, data[off..].as_ptr() as *const _, data.len() - off) };
        if n < 0 {
            return Err(Errno(unsafe { *libc::__errno_location() }));
        }
        if n == 0 {
            return Err(Errno(libc::EPIPE));
        }
        off += n as usize;
    }
    Ok(())
}

fn write_response(fd: libc::c_int, status: &str, content_type: &str, body: &[u8]) -> Result<(), Errno> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    write_all(fd, header.as_bytes())?;
    write_all(fd, body)
}

const GUI_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Props</title>
  <style>
    body { font-family: sans-serif; margin: 20px; max-width: 700px; }
    h1 { margin-bottom: 12px; }
    .row { display: grid; grid-template-columns: 220px 1fr; gap: 12px; align-items: center; margin: 8px 0; }
    .name { font-family: monospace; }
    .hint { color: #666; font-size: 12px; margin-top: 2px; }
    input[type=text], input[type=number], select { width: 100%; padding: 6px; box-sizing: border-box; }
  </style>
</head>
<body>
  <h1>Device Properties</h1>
  <div id="props"></div>
  <script>
    async function loadProps() {
      const res = await fetch('/props');
      const props = await res.json();
      render(props);
    }

    async function setProp(name, value) {
      const res = await fetch('/prop/' + encodeURIComponent(name), { method: 'PUT', body: value });
      if (!res.ok) {
        const msg = await res.text();
        alert(name + ': ' + msg);
        await loadProps();
      }
    }

    function render(props) {
      const root = document.getElementById('props');
      root.innerHTML = '';
      for (const p of props) {
        const row = document.createElement('div');
        row.className = 'row';

        const left = document.createElement('div');
        left.innerHTML = '<div class="name">' + p.name + '</div><div class="hint">' + p.type + '</div>';
        row.appendChild(left);

        let input;
        if (p.type === 'bool') {
          input = document.createElement('input');
          input.type = 'checkbox';
          input.checked = (p.value === 'true');
          input.onchange = () => setProp(p.name, input.checked ? 'true' : 'false');
        } else if (p.type === 'enum') {
          input = document.createElement('select');
          for (const v of p.values) {
            const o = document.createElement('option');
            o.value = v;
            o.textContent = v;
            o.selected = (v === p.value);
            input.appendChild(o);
          }
          input.onchange = () => setProp(p.name, input.value);
        } else if (p.type === 'int') {
          input = document.createElement('input');
          input.type = 'number';
          input.min = String(p.min);
          input.max = String(p.max);
          input.value = p.value;
          input.onchange = () => setProp(p.name, input.value);
        } else {
          input = document.createElement('input');
          input.type = 'text';
          input.minLength = p.min_len;
          input.maxLength = p.max_len;
          input.value = p.value;
          input.onchange = () => setProp(p.name, input.value);
        }

        row.appendChild(input);
        root.appendChild(row);
      }
    }

    loadProps();
  </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::parse_http_request;

    #[test]
    fn parse_chrome_like_get() {
        let req = b"GET /props HTTP/1.1\r\nHost: 192.168.50.2\r\nConnection: keep-alive\r\n\r\n";
        let parsed = parse_http_request(req).unwrap();
        assert_eq!(parsed.method, "GET");
        assert_eq!(parsed.path, "/props");
    }
}

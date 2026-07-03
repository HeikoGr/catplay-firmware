use core::sync::atomic::{AtomicI32, Ordering};
use core::time::Duration;

use crate::{SystemUtil, boot::BootUltra, dmesg};

static BOOT_TAIL_SIGNAL: AtomicI32 = AtomicI32::new(0);

impl BootUltra {
    /// Early init in PID1 mode
    pub fn boot_head() {
        let _ = SystemUtil::exec("/etc/init.d/c2a-overlay-init", &["start"]);
    }

    /// Start behaving like PID1 in a way compatible with Busybox init.
    pub fn boot_tail() {
        install_boot_tail_signal_handlers();

        loop {
            let signal = BOOT_TAIL_SIGNAL.swap(0, Ordering::Relaxed);
            if signal != 0 {
                Self::boot_tail_shutdown(signal);
            }

            let mut status: libc::c_int = 0;
            let ret = unsafe { libc::waitpid(-1, &mut status, 0) };
            if ret > 0 {
                continue;
            }

            let errno = unsafe { *libc::__errno_location() };
            if errno != libc::EINTR {
                SystemUtil::sleep(Duration::from_secs(1));
            }
        }
    }

    fn boot_tail_shutdown(signal: libc::c_int) {
        let command = match signal {
            libc::SIGUSR1 => libc::LINUX_REBOOT_CMD_HALT,
            libc::SIGUSR2 => libc::LINUX_REBOOT_CMD_POWER_OFF,
            libc::SIGTERM => libc::LINUX_REBOOT_CMD_RESTART,
            _ => {
                dmesg!("[boot] Received weird PID1 signal {signal}");
                return;
            }
        };

        dmesg!("[boot] Shutdown signal received: {signal}");
        let _ = SystemUtil::run_shell("/etc/init.d/rcK; swapoff -a; umount -a -r");
        unsafe {
            libc::sync();
            libc::syscall(
                libc::SYS_reboot,
                libc::LINUX_REBOOT_MAGIC1,
                libc::LINUX_REBOOT_MAGIC2,
                command,
                core::ptr::null::<libc::c_void>(),
            );
        }

        loop {
            SystemUtil::sleep(Duration::from_secs(1));
        }
    }
}

extern "C" fn boot_tail_signal_handler(signal: libc::c_int) {
    BOOT_TAIL_SIGNAL.store(signal, Ordering::Relaxed);
}

fn install_boot_tail_signal_handlers() {
    unsafe {
        let mut action: libc::sigaction = core::mem::zeroed();
        action.sa_sigaction = boot_tail_signal_handler as libc::sighandler_t;
        action.sa_flags = 0;
        libc::sigemptyset(&mut action.sa_mask);

        libc::sigaction(libc::SIGUSR1, &action, core::ptr::null_mut());
        libc::sigaction(libc::SIGUSR2, &action, core::ptr::null_mut());
        libc::sigaction(libc::SIGTERM, &action, core::ptr::null_mut());
    }
}

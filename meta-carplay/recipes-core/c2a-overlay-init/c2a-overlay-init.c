#define _GNU_SOURCE
#include <errno.h>
#include <dirent.h>
#include <fcntl.h>
#include <grp.h>
#include <linux/limits.h>
#include <sched.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

static void warnf(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    fprintf(stderr, "\n");
    va_end(ap);
}

static void kmsg_log(const char *msg) {
    int fd = open("/dev/kmsg", O_WRONLY | O_CLOEXEC);
    if (fd >= 0) {
        write(fd, msg, strlen(msg));
        close(fd);
    } else {
        warnf("open /dev/kmsg failed: %s", strerror(errno));
    }
}

static int mkdir_p(const char *path, mode_t mode) {
    char tmp[PATH_MAX];
    char *p = NULL;
    size_t len;

    if (!path || !*path) return -1;
    strncpy(tmp, path, sizeof(tmp));
    tmp[sizeof(tmp)-1] = '\0';
    len = strlen(tmp);
    if (tmp[len-1] == '/') tmp[len-1] = '\0';

    for (p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            if (mkdir(tmp, mode) != 0) {
                if (errno != EEXIST) return -1;
            }
            *p = '/';
        }
    }
    if (mkdir(tmp, mode) != 0) {
        if (errno != EEXIST) return -1;
    }
    return 0;
}

static int cmdline_has_flag(const char *flag) {
    char buf[4096];
    ssize_t n;
    int fd = open("/proc/cmdline", O_RDONLY | O_CLOEXEC);

    if (fd < 0) return 0;
    n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) return 0;
    buf[n] = '\0';

    char *saveptr = NULL;
    for (char *token = strtok_r(buf, " \t\r\n", &saveptr);
         token; token = strtok_r(NULL, " \t\r\n", &saveptr)) {
        if (strcmp(token, flag) == 0) return 1;
    }
    return 0;
}

static int mount_tmpfs(const char *path, const char *options) {
    if (mkdir_p(path, 0755) != 0) return -1;
    return mount("tmpfs", path, "tmpfs", MS_NOSUID | MS_NODEV, options);
}

static int copy_metadata(int dstfd, const char *name, const struct stat *st,
                         int symlink) {
    if (!symlink) {
        if (fchmodat(dstfd, name, st->st_mode & 07777, 0) != 0 &&
            errno != EOPNOTSUPP)
            return -1;
    }
    if (fchownat(dstfd, name, st->st_uid, st->st_gid,
                 symlink ? AT_SYMLINK_NOFOLLOW : 0) != 0 && errno != EPERM)
        return -1;
    return 0;
}

static int copy_node(int srcfd, int dstfd, const char *name,
                     const struct stat *st);

static int copy_directory(int srcfd, int dstfd) {
    int result = 0;
    DIR *dir = fdopendir(srcfd);
    if (!dir) {
        close(srcfd);
        return -1;
    }
    for (;;) {
        struct dirent *entry = readdir(dir);
        if (!entry)
            break;
        if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, ".."))
            continue;
        struct stat st;
        if (fstatat(dirfd(dir), entry->d_name, &st, AT_SYMLINK_NOFOLLOW) != 0) {
            result = -1;
            continue;
        }
        if (copy_node(dirfd(dir), dstfd, entry->d_name, &st) != 0)
            result = -1;
    }
    if (closedir(dir) != 0)
        result = -1;
    return result;
}

static int copy_regular(int srcfd, int dstfd, const char *name,
                        const struct stat *st) {
    int in = openat(srcfd, name, O_RDONLY | O_CLOEXEC | O_NOFOLLOW);
    if (in < 0)
        return -1;
    int out = openat(dstfd, name, O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC,
                     st->st_mode & 07777);
    if (out < 0) {
        close(in);
        return -1;
    }
    char buf[16384];
    int result = 0;
    for (;;) {
        ssize_t n = read(in, buf, sizeof(buf));
        if (n == 0)
            break;
        if (n < 0) {
            if (errno == EINTR)
                continue;
            result = -1;
            break;
        }
        ssize_t off = 0;
        while (off < n) {
            ssize_t written = write(out, buf + off, (size_t)(n - off));
            if (written < 0) {
                if (errno == EINTR)
                    continue;
                result = -1;
                break;
            }
            off += written;
        }
        if (result != 0)
            break;
    }
    if (fchmod(out, st->st_mode & 07777) != 0)
        result = -1;
    if (fchown(out, st->st_uid, st->st_gid) != 0 && errno != EPERM)
        result = -1;
    close(out);
    close(in);
    return result;
}

static int copy_node(int srcfd, int dstfd, const char *name,
                     const struct stat *st) {
    if (S_ISREG(st->st_mode))
        return copy_regular(srcfd, dstfd, name, st);

    if (S_ISLNK(st->st_mode)) {
        char target[PATH_MAX];
        ssize_t len = readlinkat(srcfd, name, target, sizeof(target) - 1);
        if (len < 0)
            return -1;
        target[len] = '\0';
        if (symlinkat(target, dstfd, name) != 0)
            return -1;
        return copy_metadata(dstfd, name, st, 1);
    }

    if (S_ISDIR(st->st_mode)) {
        if (mkdirat(dstfd, name, st->st_mode & 07777) != 0 && errno != EEXIST)
            return -1;
        int src_child = openat(srcfd, name, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
        int dst_child = openat(dstfd, name, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
        if (src_child < 0 || dst_child < 0) {
            if (src_child >= 0) close(src_child);
            if (dst_child >= 0) close(dst_child);
            return -1;
        }
        int result = copy_directory(src_child, dst_child);
        if (copy_metadata(dstfd, name, st, 0) != 0)
            result = -1;
        close(dst_child);
        return result;
    }

    if (S_ISFIFO(st->st_mode)) {
        if (mkfifoat(dstfd, name, st->st_mode & 07777) != 0)
            return -1;
        return copy_metadata(dstfd, name, st, 0);
    }

    return 0;
}

/* Equivalent to copying source/. into target, without cp, BusyBox, fork or
 * exec. All traversal and file operations stay in this process. */
static int copy_dir_contents(const char *source, const char *target) {
    int srcfd = open(source, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    int dstfd = open(target, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    if (srcfd < 0 || dstfd < 0) {
        if (srcfd >= 0) close(srcfd);
        if (dstfd >= 0) close(dstfd);
        return -1;
    }
    int result = copy_directory(srcfd, dstfd);
    close(dstfd);
    return result;
}

int main(int argc, char *argv[]) {
    if (argc < 2 || strcmp(argv[1], "start") != 0) {
        fprintf(stderr, "Usage: %s start\n", argv[0]);
        return 0;
    }

    kmsg_log("[c2a-overlay-init] Starting (fast)!\n");

    // 1) hostname
    {
        char buf[256];
        ssize_t n = 0;
        int fd = open("/etc/hostname", O_RDONLY);
        if (fd >= 0) {
            n = read(fd, buf, sizeof(buf)-1);
            close(fd);
            if (n > 0) {
                // trim newline
                buf[n] = '\0';
                char *nl = strchr(buf, '\n');
                if (nl) *nl = '\0';
                if (sethostname(buf, strlen(buf)) != 0) {
                    warnf("sethostname failed: %s", strerror(errno));
                }
            }
        } else {
            // not fatal
            warnf("could not open /etc/hostname: %s", strerror(errno));
        }
    }

    // 2) remount / rw
    if (mount(NULL, "/", NULL, MS_REMOUNT, "rw") != 0) {
        // non-fatal
        warnf("remount / rw failed (continuing): %s", strerror(errno));
    }

    // 3) sysfs / proc / configfs
    if (mount("proc", "/proc", "proc", 0, NULL) != 0) {
        warnf("mount proc failed: %s", strerror(errno));
    }
    if (mount("sysfs", "/sys", "sysfs", 0, NULL) != 0) {
        warnf("mount sysfs failed: %s", strerror(errno));
    }
    if (mount("configfs", "/sys/kernel/config", "configfs", 0, NULL) != 0) {
        warnf("mount configfs failed: %s", strerror(errno));
    }

    // 4) alignment sysctl equivalent: echo "3" > /proc/cpu/alignment
    {
        int fd = open("/proc/cpu/alignment", O_WRONLY);
        if (fd >= 0) {
            const char *v = "3\n";
            write(fd, v, strlen(v));
            close(fd);
        } else {
            warnf("cannot set /proc/cpu/alignment: %s", strerror(errno));
        }
    }

    // 5) mount tmpfs on /run
    if (mkdir_p("/run", 0755) != 0) {
        warnf("mkdir /run failed: %s", strerror(errno));
    }
    if (mount("tmpfs", "/run", "tmpfs", 0, NULL) != 0) {
        warnf("mount tmpfs /run failed: %s", strerror(errno));
    }

    int no_overlay = cmdline_has_flag("c2a_no_ovl");

    // 6) Either mount the normal overlay or bind the original root and use
    // explicit tmpfs mounts for the writable state needed by daemons.
    mkdir_p("/run/newroot", 0755);

    if (no_overlay) {
        kmsg_log("[c2a-overlay-init] c2a_no_ovl: disabling overlayfs\n");
        mkdir_p("/run/etc-copy", 0755);
        {
            int copy_status = copy_dir_contents("/etc/.", "/run/etc-copy");
            char msg[128];
            snprintf(msg, sizeof(msg),
                     "[c2a-overlay-init] c2a_no_ovl: pre-pivot /etc copy status=%d\n",
                     copy_status);
            kmsg_log(msg);
        }
        if (mount("/", "/run/newroot", NULL, MS_BIND, NULL) != 0) {
            warnf("bind-mount root for c2a_no_ovl failed: %s", strerror(errno));
        }
    } else {
        mkdir_p("/run/upper", 0755);
        mkdir_p("/run/work", 0755);
        const char *overlay_opts = "lowerdir=/,upperdir=/run/upper,workdir=/run/work";
        if (mount("overlay", "/run/newroot", "overlay", 0, overlay_opts) != 0) {
            warnf("mount overlay failed: %s", strerror(errno));
        }
    }

    // prepare old_root
    mkdir_p("/run/newroot/old_root", 0755);

    kmsg_log("[c2a-overlay-init] Performing pivot_root\n");

    // 7) pivot_root: syscall
    // pivot_root(new_root, put_old)
    if (syscall(SYS_pivot_root, "/run/newroot", "/run/newroot/old_root") != 0) {
        warnf("pivot_root failed: %s", strerror(errno));
    } else {
        // move mounts: old_root/dev -> /dev, old_root/sys -> /sys, old_root/proc -> /proc
        // if (mount("/old_root/dev", "/dev", NULL, MS_MOVE, NULL) != 0) {
        //     // try with run/newroot/old_root path if needed
        //     warnf("move /old_root/dev -> /dev failed: %s", strerror(errno));
        // }
        if (mount("/old_root/sys", "/sys", NULL, MS_MOVE, NULL) != 0) {
            warnf("move /old_root/sys -> /sys failed: %s", strerror(errno));
        }
        if (mount("/old_root/proc", "/proc", NULL, MS_MOVE, NULL) != 0) {
            warnf("move /old_root/proc -> /proc failed: %s", strerror(errno));
        }
    }

    if (no_overlay) {
        if (mount_tmpfs("/etc", "mode=0755,size=2M") != 0) {
            warnf("mount tmpfs /etc failed: %s", strerror(errno));
        } else {
            int restore_status = copy_dir_contents("/run/etc-copy/.", "/etc");
            char msg[128];
            snprintf(msg, sizeof(msg),
                     "[c2a-overlay-init] c2a_no_ovl: /etc restore status=%d\n",
                     restore_status);
            kmsg_log(msg);
        }
        if (mount_tmpfs("/var/lib", "mode=0755,size=4M") != 0)
            warnf("mount tmpfs /var/lib failed: %s", strerror(errno));
        if (mount_tmpfs("/var/cache", "mode=0755,size=2M") != 0)
            warnf("mount tmpfs /var/cache failed: %s", strerror(errno));
        if (mount_tmpfs("/var/volatile", "mode=0755,size=8M") != 0)
            warnf("mount tmpfs /var/volatile failed: %s", strerror(errno));
    }

    if (mount("devtmpfs", "/dev", "devtmpfs", MS_NOSUID, "mode=0755") != 0) {
        warnf("mount devtmpfs failed: %s", strerror(errno));
    }

    // 8) devpts: create /dev/pts and mount devpts with gid=5,mode=620
    if (mkdir_p("/dev/pts", 0755) != 0) {
        warnf("mkdir /dev/pts failed: %s", strerror(errno));
    }
    if (mount("devpts", "/dev/pts", "devpts", 0, "gid=5,mode=620") != 0) {
        warnf("mount devpts failed: %s", strerror(errno));
    }

    // 9) /var/volatile: mount tmpfs for lib, log, tmp

    // lib: runtime state (no limit)
    if (mkdir_p("/var/volatile/lib", 0755) != 0) {
        warnf("mkdir /var/volatile/lib failed: %s", strerror(errno));
    }
    if (mount("tmpfs", "/var/volatile/lib", "tmpfs",
            MS_NOSUID | MS_NODEV,
            "mode=0755") != 0) {
        warnf("mount tmpfs on /var/volatile/lib failed: %s", strerror(errno));
    }

    // log: capped to 4M
    if (mkdir_p("/var/volatile/log", 0755) != 0) {
        warnf("mkdir /var/volatile/log failed: %s", strerror(errno));
    }
    if (mount("tmpfs", "/var/volatile/log", "tmpfs",
            MS_NOSUID | MS_NODEV,
            "mode=0755,size=4M") != 0) {
        warnf("mount tmpfs on /var/volatile/log failed: %s", strerror(errno));
    }

    // tmp: like /tmp (sticky), no limit
    if (mkdir_p("/var/volatile/tmp", 01777) != 0) {
        warnf("mkdir /var/volatile/tmp failed: %s", strerror(errno));
    }
    if (mount("tmpfs", "/var/volatile/tmp", "tmpfs",
            MS_NOSUID | MS_NODEV,
            "mode=1777") != 0) {
        warnf("mount tmpfs on /var/volatile/tmp failed: %s", strerror(errno));
    }

    // 10) dump kernel log, capped to 128K
    {
        enum { SYSLOG_ACTION_READ_ALL = 3, DMESG_BUFSIZE = 131072 };
        static char dmesg_buf[DMESG_BUFSIZE];

        int len = syscall(SYS_syslog, SYSLOG_ACTION_READ_ALL, dmesg_buf, sizeof(dmesg_buf));
        if (len < 0) {
            warnf("reading kernel log failed (ignored): %s", strerror(errno));
        } else {
            int fd = open("/var/log/dmesg", O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC, 0644);
            if (fd < 0) {
                warnf("open /var/log/dmesg failed (ignored): %s", strerror(errno));
            } else {
                ssize_t off = 0;
                while (off < len) {
                    ssize_t written = write(fd, dmesg_buf + off, len - off);
                    if (written < 0) {
                        if (errno == EINTR) continue;
                        warnf("write /var/log/dmesg failed (ignored): %s", strerror(errno));
                        break;
                    }
                    off += written;
                }
                close(fd);
            }
        }
    }

    // wait until devtmpfs is fully populated
    struct stat st;
    for (int i = 0; i < 200; i++) {
        if (stat("/dev/ptmx", &st) == 0 &&
            stat("/dev/null", &st) == 0 &&
            stat("/dev/tty", &st) == 0)
            break;
        usleep(10000); // 10 ms
    }
    
    // final kmsg message
    kmsg_log("[c2a-overlay-init] Finished mounting C2A overlays (fast)!\n");

    return 0;
}

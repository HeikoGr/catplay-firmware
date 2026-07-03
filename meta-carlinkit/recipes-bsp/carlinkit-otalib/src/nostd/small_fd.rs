extern crate alloc;

use core::{ffi::c_void, ptr};

use alloc::sync::Arc;
use libc::{
    MAP_PRIVATE, MAP_SHARED, MS_SYNC, PROT_READ, PROT_WRITE, S_IRWXU, c_int, close, fstat, mmap, msync, munmap, off_t, open, read, size_t,
    write,
};

pub struct FdGuard {
    pub fd: c_int,
}

impl FdGuard {
    pub fn new(fd: c_int) -> Self {
        Self { fd }
    }
}

impl Drop for FdGuard {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

pub struct SmallFd {
    fd: Arc<FdGuard>,
}

impl SmallFd {
    pub fn open(path: &str) -> Result<Self, &'static str> {
        Self::open_with_flags(path, libc::O_RDWR)
    }

    pub fn open_readonly(path: &str) -> Result<Self, &'static str> {
        Self::open_with_flags(path, libc::O_RDONLY)
    }

    pub fn open_writeonly(path: &str) -> Result<Self, &'static str> {
        Self::open_with_flags(path, libc::O_WRONLY)
    }

    pub fn open_writeonly_or_create(path: &str) -> Result<Self, &'static str> {
        match Self::open_with_flags(path, libc::O_WRONLY) {
            Ok(fd) => Ok(fd),
            Err(_) if Self::errno() == libc::ENOENT => Self::open_with_flags(path, libc::O_WRONLY | libc::O_CREAT),
            Err(err) => Err(err),
        }
    }

    pub fn create(path: &str) -> Result<Self, &'static str> {
        Self::open_with_flags(path, libc::O_RDWR | libc::O_CREAT)
    }

    fn open_with_flags(path: &str, flags: c_int) -> Result<Self, &'static str> {
        let mut buf = [0u8; 1024];
        buf[..path.len()].copy_from_slice(path.as_bytes());
        assert!(path.len() < buf.len());

        let fd = unsafe { open(buf.as_ptr() as *const _, flags, S_IRWXU) };
        if fd < 0 {
            return Err("failed to open fd");
        }
        Ok(Self {
            fd: Arc::new(FdGuard::new(fd)),
        })
    }

    fn errno() -> c_int {
        unsafe { *libc::__errno_location() }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let n: isize = unsafe { read(self.fd.fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if n < 0 {
            return Err("failed to read from fd");
        }

        Ok(n as usize)
    }

    pub fn raw_fd(&self) -> c_int {
        self.fd.fd
    }

    #[allow(dead_code)]
    pub fn write(&self, buf: &[u8]) -> Result<usize, &'static str> {
        let n: isize = unsafe { write(self.fd.fd, buf.as_ptr() as *mut _, buf.len()) };
        if n < 0 {
            return Err("failed to write to fd");
        }

        Ok(n as usize)
    }

    pub fn mmap_readonly(&self, offset: usize, len: usize) -> Result<MmapGuard, &'static str> {
        let addr = unsafe { mmap(ptr::null_mut(), len as size_t, PROT_READ, MAP_SHARED, self.fd.fd, offset as off_t) };
        if addr == libc::MAP_FAILED {
            return Err("mmap failed");
        }

        Ok(MmapGuard::new(self.fd.clone(), addr, len))
    }

    pub fn mmap(&self, offset: usize, len: usize) -> Result<MmapGuard, &'static str> {
        let addr = unsafe {
            mmap(
                ptr::null_mut(),
                len as size_t,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                self.fd.fd,
                offset as off_t,
            )
        };
        if addr == libc::MAP_FAILED {
            return Err("mmap failed");
        }

        Ok(MmapGuard::new(self.fd.clone(), addr, len))
    }

    pub fn mmap_priv(&self, offset: usize, len: usize) -> Result<MmapGuard, &'static str> {
        let addr = unsafe {
            mmap(
                ptr::null_mut(),
                len as size_t,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE,
                self.fd.fd,
                offset as off_t,
            )
        };
        if addr == libc::MAP_FAILED {
            return Err("mmap failed");
        }

        Ok(MmapGuard::new(self.fd.clone(), addr, len))
    }

    pub fn truncate(&self, length: usize) -> Result<(), &'static str> {
        let ret = unsafe { libc::ftruncate(self.fd.fd, length as off_t) };
        if ret < 0 {
            return Err("failed to truncate (prealloc) file");
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn stat(&self) -> Result<libc::stat, &'static str> {
        let mut st: libc::stat = unsafe { core::mem::zeroed() };
        if unsafe { fstat(self.fd.fd, &mut st) } != 0 {
            return Err("failed to stat fd");
        }
        Ok(st)
    }
}

pub struct MmapGuard {
    _fd: Arc<FdGuard>,
    addr: *mut c_void,
    map_len: usize,
}

impl MmapGuard {
    pub fn new(fd: Arc<FdGuard>, addr: *mut c_void, map_len: usize) -> Self {
        Self { _fd: fd, addr, map_len }
    }

    pub fn msync(&self) -> Result<(), &'static str> {
        let val = unsafe { msync(self.addr, self.map_len, MS_SYNC) };
        if val != 0 {
            return Err("msync failed");
        }
        Ok(())
    }

    pub fn mem(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.addr as *mut u8, self.map_len) }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), &'static str> {
        self.mem()[offset..offset + data.len()].copy_from_slice(data);
        self.msync()
    }

    pub fn read(&mut self, offset: usize, data: &mut [u8]) -> Result<(), &'static str> {
        data.copy_from_slice(&self.mem()[offset..offset + data.len()]);

        Ok(())
    }

    pub fn write_if_different(&mut self, offset: usize, data: &[u8], block_size: usize) -> Result<(), &'static str> {
        let mem = self.mem();

        let mut written = false;
        let mut pos = 0;

        while pos < data.len() {
            let block_off = offset + pos;
            let remain = data.len() - pos;
            let chunk_len = remain.min(block_size);

            let old_chunk = &mem[block_off..block_off + chunk_len];
            let new_chunk = &data[pos..pos + chunk_len];

            if old_chunk != new_chunk {
                mem[block_off..block_off + chunk_len].copy_from_slice(new_chunk);
                written = true;
            }

            pos += chunk_len;
        }

        if written { self.msync() } else { Ok(()) }
    }
}

impl Drop for MmapGuard {
    fn drop(&mut self) {
        let _ = self.msync();
        let _ = unsafe { munmap(self.addr, self.map_len) };
    }
}

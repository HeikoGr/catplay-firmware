extern crate alloc;

use core::ffi::CStr;

use alloc::string::{String, ToString};
use heapless::{CString, String as HString, Vec as HVec};

use crate::nostd::{ModulesInfo, SmallFd};

pub struct ModprobeUtil;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModprobeError {
    PushStrFailed,
    ModuleNotFound(String),
    ModuleCompressed(String),
    DependencyNotFound { dependency: String, for_module: String },
    ModuleLoadFailed { module: String, error: &'static str },
}

impl ModprobeUtil {
    pub fn modprobe(module: &str) -> Result<(), ModprobeError> {
        Self::modprobe_with_params(module, "")
    }

    pub fn for_each_dependency_module_path(
        module: &str,
        mut f: impl FnMut(&str) -> Result<(), ModprobeError>,
    ) -> Result<(), ModprobeError> {
        Self::for_each_dependency_module(module, |_, path| f(path))
    }

    pub fn modprobe_with_params(module: &str, params: &str) -> Result<(), ModprobeError> {
        let module_name = Self::normalize_module_name(module)?;

        Self::for_each_dependency_module(module, |dep_name, abs_path| {
            let module_params = if dep_name == module_name.as_str() { params } else { "" };
            Self::finit_module(dep_name, abs_path, module_params)
        })
    }

    fn for_each_dependency_module(module: &str, mut f: impl FnMut(&str, &str) -> Result<(), ModprobeError>) -> Result<(), ModprobeError> {
        let module_name = Self::normalize_module_name(module)?;
        let modules_root = Self::modules_root_dir()?;
        let info = Self::load_modules_info(modules_root.as_str())?;

        if info.is_builtin(module_name.as_str()) {
            return Ok(());
        }
        if info.resolve_path(module_name.as_str()).is_none() {
            return Err(ModprobeError::ModuleNotFound(module_name.as_str().to_string()));
        }

        Self::validate_dependency_paths(&info, module_name.as_str())?;

        let order = info.resolve_load_order(module_name.as_str()).map_err(|_| ModprobeError::ModuleLoadFailed {
            module: module_name.as_str().to_string(),
            error: "dependency cycle detected",
        })?;

        for dep_name in order {
            if info.is_builtin(dep_name.as_str()) {
                continue;
            }

            let rel_path = info
                .resolve_path(dep_name.as_str())
                .ok_or_else(|| ModprobeError::ModuleNotFound(dep_name.clone()))?;
            if rel_path.contains(".ko.") {
                return Err(ModprobeError::ModuleCompressed(dep_name));
            }
            let abs_path = Self::join_path::<512>(modules_root.as_str(), rel_path.as_str())?;
            f(dep_name.as_str(), abs_path.as_str())?;
        }

        Ok(())
    }

    fn normalize_module_name(module: &str) -> Result<HString<128>, ModprobeError> {
        let raw_name = module.rsplit('/').next().unwrap_or(module);
        let name = if let Some(pos) = raw_name.find(".ko") {
            &raw_name[..pos]
        } else {
            raw_name
        };
        if name.is_empty() {
            return Err(ModprobeError::ModuleNotFound(String::new()));
        }

        let mut out = HString::<128>::new();
        out.push_str(name).map_err(|_| ModprobeError::PushStrFailed)?;
        Ok(out)
    }

    fn modules_root_dir() -> Result<HString<256>, ModprobeError> {
        let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
        if unsafe { libc::uname(&mut uts as *mut _) } < 0 {
            let errno = unsafe { *libc::__errno_location() };
            return Err(ModprobeError::ModuleLoadFailed {
                module: "uname".to_string(),
                error: Self::errno_to_static_str(errno),
            });
        }

        let release = unsafe { CStr::from_ptr(uts.release.as_ptr()) }.to_str().map_err(|_| ModprobeError::PushStrFailed)?;

        let mut lib_modules = HString::<256>::new();
        lib_modules.push_str("/lib/modules/").map_err(|_| ModprobeError::PushStrFailed)?;
        lib_modules.push_str(release).map_err(|_| ModprobeError::PushStrFailed)?;
        if Self::path_exists(lib_modules.as_str())? {
            return Ok(lib_modules);
        }

        let mut usr_lib_modules = HString::<256>::new();
        usr_lib_modules.push_str("/usr/lib/modules/").map_err(|_| ModprobeError::PushStrFailed)?;
        usr_lib_modules.push_str(release).map_err(|_| ModprobeError::PushStrFailed)?;
        if Self::path_exists(usr_lib_modules.as_str())? {
            return Ok(usr_lib_modules);
        }

        Err(ModprobeError::ModuleNotFound(release.to_string()))
    }

    fn load_modules_info(modules_root: &str) -> Result<ModulesInfo, ModprobeError> {
        let dep_path = Self::join_path::<320>(modules_root, "modules.dep")?;
        let builtin_path = Self::join_path::<320>(modules_root, "modules.builtin")?;

        let mut info = ModulesInfo::new();
        Self::for_each_file_line(dep_path.as_str(), |line| {
            info.feed_modules_dep_line(line);
        })?;

        if Self::path_exists(builtin_path.as_str())? {
            Self::for_each_file_line(builtin_path.as_str(), |line| {
                info.feed_modules_builtin_line(line);
            })?;
        }

        Ok(info)
    }

    fn for_each_file_line<F>(path: &str, mut f: F) -> Result<(), ModprobeError>
    where
        F: FnMut(&[u8]),
    {
        let fd = SmallFd::open_readonly(path).map_err(|_| ModprobeError::ModuleNotFound(path.to_string()))?;
        let mut chunk = [0u8; 1024];
        let mut line = HVec::<u8, 2048>::new();

        loop {
            let n = fd.read(&mut chunk).map_err(|_| ModprobeError::ModuleNotFound(path.to_string()))?;
            if n == 0 {
                break;
            }

            for b in &chunk[..n] {
                if *b == b'\n' {
                    if let Some((&b'\r', prefix)) = line.split_last() {
                        f(prefix);
                    } else {
                        f(line.as_slice());
                    }
                    line.clear();
                    continue;
                }

                if line.push(*b).is_err() {
                    return Err(ModprobeError::PushStrFailed);
                }
            }
        }

        if let Some((&b'\r', prefix)) = line.split_last() {
            if !prefix.is_empty() {
                f(prefix);
            }
        } else if !line.is_empty() {
            f(line.as_slice());
        }

        Ok(())
    }

    fn join_path<const N: usize>(base: &str, name: &str) -> Result<HString<N>, ModprobeError> {
        let mut out = HString::<N>::new();
        out.push_str(base).map_err(|_| ModprobeError::PushStrFailed)?;
        if !base.ends_with('/') {
            out.push('/').map_err(|_| ModprobeError::PushStrFailed)?;
        }
        out.push_str(name).map_err(|_| ModprobeError::PushStrFailed)?;
        Ok(out)
    }

    fn path_exists(path: &str) -> Result<bool, ModprobeError> {
        let c_path = Self::to_cstring::<256>(path)?;
        let ret = unsafe { libc::access(c_path.as_ptr(), libc::F_OK) };
        Ok(ret == 0)
    }

    fn to_cstring<const N: usize>(s: &str) -> Result<CString<N>, ModprobeError> {
        let mut c = CString::<N>::new();
        if c.extend_from_bytes(s.as_bytes()).is_err() {
            return Err(ModprobeError::PushStrFailed);
        }
        Ok(c)
    }

    fn finit_module(module_name: &str, path: &str, params: &str) -> Result<(), ModprobeError> {
        unsafe {
            let fd = SmallFd::open_readonly(path).map_err(|_| ModprobeError::ModuleNotFound(path.to_string()))?;
            let params_c = Self::to_cstring::<512>(params)?;

            let ret = libc::syscall(libc::SYS_finit_module as libc::c_long, fd.raw_fd(), params_c.as_ptr(), 0usize);
            let errno = *libc::__errno_location();

            if ret == 0 || errno == libc::EEXIST {
                Ok(())
            } else {
                Err(ModprobeError::ModuleLoadFailed {
                    module: module_name.to_string(),
                    error: Self::errno_to_static_str(errno),
                })
            }
        }
    }

    fn errno_to_static_str(errno: i32) -> &'static str {
        unsafe {
            let ptr = libc::strerror(errno);
            if ptr.is_null() {
                return "Unknown error";
            }

            match CStr::from_ptr(ptr).to_str() {
                Ok(s) => core::mem::transmute::<&str, &'static str>(s),
                Err(_) => "Unknown error",
            }
        }
    }

    fn validate_dependency_paths(info: &ModulesInfo, root: &str) -> Result<(), ModprobeError> {
        let mut stack = HVec::<HString<128>, 512>::new();
        let mut visited = HVec::<HString<128>, 512>::new();

        let mut root_hs = HString::<128>::new();
        root_hs.push_str(root).map_err(|_| ModprobeError::PushStrFailed)?;
        stack.push(root_hs).map_err(|_| ModprobeError::PushStrFailed)?;

        while let Some(owner) = stack.pop() {
            if visited.iter().any(|v| v == &owner) {
                continue;
            }
            visited.push(owner.clone()).map_err(|_| ModprobeError::PushStrFailed)?;

            if info.is_builtin(owner.as_str()) {
                continue;
            }
            if info.resolve_path(owner.as_str()).is_none() {
                return Err(ModprobeError::ModuleNotFound(owner.as_str().to_string()));
            }

            let Some(deps) = info.deps_of(owner.as_str()) else {
                continue;
            };

            for dep in deps {
                if info.is_builtin(dep.as_str()) {
                    continue;
                }
                if info.resolve_path(dep.as_str()).is_none() {
                    return Err(ModprobeError::DependencyNotFound {
                        dependency: dep.clone(),
                        for_module: owner.as_str().to_string(),
                    });
                }

                let mut dep_hs = HString::<128>::new();
                dep_hs.push_str(dep.as_str()).map_err(|_| ModprobeError::PushStrFailed)?;
                stack.push(dep_hs).map_err(|_| ModprobeError::PushStrFailed)?;
            }
        }

        Ok(())
    }
}

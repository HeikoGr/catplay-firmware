extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

pub struct ModulesInfo {
    pub deps: BTreeMap<String, Vec<String>>,
    pub builtin: BTreeSet<String>,
    pub paths: BTreeMap<String, String>,
}

impl ModulesInfo {
    pub const fn new() -> Self {
        Self {
            deps: BTreeMap::new(),
            builtin: BTreeSet::new(),
            paths: BTreeMap::new(),
        }
    }

    fn short_name(path: &[u8]) -> String {
        // find last '/'
        let name = match path.iter().rposition(|&b| b == b'/') {
            Some(pos) => &path[pos + 1..],
            None => path,
        };

        // Handle plain and compressed module filenames:
        // foo.ko, foo.ko.xz, foo.ko.gz, foo.ko.zst -> foo
        let name = if let Some(pos) = name.windows(3).position(|w| w == b".ko") {
            &name[..pos]
        } else {
            name
        };
        String::from_utf8_lossy(name).into_owned()
    }

    pub fn resolve_path(&self, name: &str) -> Option<&String> {
        self.paths.get(name)
    }

    /// `path/to/module.ko: dep1.ko dep2.ko`
    pub fn feed_modules_dep_line(&mut self, line: &[u8]) {
        let Some(colon_idx) = line.iter().position(|&b| b == b':') else {
            return;
        };

        let (mod_path, deps_part) = line.split_at(colon_idx);
        let deps_part = &deps_part[1..]; // skip ':'

        let name = Self::short_name(mod_path);
        let mut deps_vec = Vec::new();

        // Dependencies seperated by a space
        for word in deps_part.split(|&b| b == b' ' || b == b'\t') {
            if word.is_empty() {
                continue;
            }
            let dep_name = Self::short_name(word);
            if dep_name.is_empty() {
                continue;
            }
            deps_vec.push(dep_name);
        }

        let path_str = String::from_utf8_lossy(mod_path).into_owned();
        self.paths.insert(name.clone(), path_str);
        self.deps.insert(name.clone(), deps_vec);
    }

    /// `kernel/fs/ext4/ext4.ko`
    pub fn feed_modules_builtin_line(&mut self, line: &[u8]) {
        let name = Self::short_name(line);
        if name.is_empty() {
            return;
        }
        self.builtin.insert(name);
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtin.contains(name) // .iter().any(|n| n.as_str() == name)
    }

    pub fn deps_of(&self, name: &str) -> Option<&Vec<String>> {
        self.deps.get(name)
    }

    pub fn resolve_load_order(&self, root: &str) -> Result<Vec<String>, &'static str> {
        use alloc::collections::BTreeMap as Map;

        const MARK_UNVISITED: u8 = 0;
        const MARK_VISITING: u8 = 1;
        const MARK_DONE: u8 = 2;

        let mut marks: Map<String, u8> = Map::new();
        let mut out_order: Vec<String> = Vec::new();
        let mut stack: Vec<(String, usize)> = Vec::new(); // (module, next_dep_index)

        // push root
        marks.insert(root.to_string(), MARK_UNVISITED);
        stack.push((root.to_string(), 0));

        while let Some((cur, idx)) = stack.last_mut() {
            let cur_name = cur.clone();

            // if builtin - finish immediately
            if self.is_builtin(&cur_name) {
                // mark done and pop
                marks.insert(cur_name.clone(), MARK_DONE);
                stack.pop();
                continue;
            }

            // get deps vector (may be empty)
            let deps_opt = self.deps.get(&cur_name);

            // mark visiting if unvisited
            if *marks.get(&cur_name).unwrap_or(&MARK_UNVISITED) == MARK_UNVISITED {
                marks.insert(cur_name.clone(), MARK_VISITING);
            }

            let deps = match deps_opt {
                Some(v) => v,
                None => {
                    // no deps entry -> treat as leaf -> add to order and pop
                    marks.insert(cur_name.clone(), MARK_DONE);
                    if !out_order.iter().any(|s| s == &cur_name) {
                        out_order.push(cur_name.clone());
                    }
                    stack.pop();
                    continue;
                }
            };

            if *idx >= deps.len() {
                // processed all deps -> mark done, append to order, pop
                marks.insert(cur_name.clone(), MARK_DONE);
                if !out_order.iter().any(|s| s == &cur_name) {
                    out_order.push(cur_name.clone());
                }
                stack.pop();
                continue;
            }

            // examine dep at idx
            let dep_name = deps[*idx].clone();
            *idx += 1;

            match *marks.get(&dep_name).unwrap_or(&0) {
                MARK_UNVISITED => {
                    // unvisited -> push
                    marks.insert(dep_name.clone(), 0);
                    stack.push((dep_name.clone(), 0));
                    continue;
                }
                MARK_VISITING => {
                    // visiting -> cycle
                    return Err("dependency cycle detected");
                }
                MARK_DONE => {
                    // already done -> continue; loop will proceed
                    continue;
                }
                _ => unreachable!(),
            }
        }

        Ok(out_order)
    }
}

#[test]
fn test_modules_info() {
    let mut repo = ModulesInfo::new();

    let dep_file = [
        "kernel/crypto/gcm.ko:",
        "kernel/crypto/ccm.ko:",
        "kernel/crypto/ghash-generic.ko: kernel/lib/crypto/gf128mul.ko",
        "kernel/lib/crypto/libarc4.ko:",
        "kernel/lib/crypto/gf128mul.ko: kernel/crypto/crc7.ko kernel/crypto/ccm.ko",
        "kernel/lib/crc7.ko:",
    ];

    let builtin_file = ["kernel/crypto/gcm.ko", "kernel/crypto/ccm.ko"];

    for line in dep_file {
        repo.feed_modules_dep_line(line.as_bytes());
    }

    for line in builtin_file {
        repo.feed_modules_builtin_line(line.as_bytes());
    }

    assert_eq!(repo.deps_of("ghash-generic").unwrap().as_slice(), &["gf128mul"]);
    assert!(repo.is_builtin("gcm"));
    assert!(repo.is_builtin("ccm"));
    assert!(!repo.is_builtin("crc7"));

    let recursive_dep = [
        "kernel/test1.ko: kernel/test2.ko",
        "kernel/test2.ko: kernel/test3.ko",
        "kernel/test3.ko: kernel/test1.ko",
    ];

    for line in recursive_dep {
        repo.feed_modules_dep_line(line.as_bytes());
    }

    assert_eq!(
        repo.resolve_load_order("ghash-generic").unwrap().as_slice(),
        ["crc7", "gf128mul", "ghash-generic"]
    );

    assert!(repo.resolve_load_order("test1").is_err());
    assert!(repo.resolve_load_order("test2").is_err());
    assert!(repo.resolve_load_order("test3").is_err());

    assert_eq!(repo.resolve_path("test1").unwrap(), "kernel/test1.ko");
}

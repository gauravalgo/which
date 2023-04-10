extern crate libc;
#[cfg(test)]
extern crate tempdir;
#[cfg(unix)]
use std::ffi::CString;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{env, fs};

pub fn is_exist(binpath: &Path) -> bool {
    match fs::metadata(binpath).map(|metadata| metadata.is_file()) {
        Ok(true) => true,
        _ => false,
    }
}
//return true is path is executable
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    CString::new(path.as_os_str().as_bytes())
        .and_then(|c| Ok(unsafe { libc::access(c.as_ptr(), libc::X_OK) == 0 }))
        .unwrap_or(false)
}
#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    true
}
///Find 'binary_name' in the path list 'paths', using 'cwd' to resolve relative paths.
pub fn which_in<T, U>(binary_name: T, paths: Option<U>, cwd: V) -> Result<PathBuf, &'static str>
where
    T: AsRef<OsStr>,
    U: AsRef<OsStr>,
{
    let path_buf = paths.and_then(|paths| -> Option<PathBuf> {
        for path in env::split_paths(&paths) {
            let bin_path = path.join(binary_name.as_ref());
            if is_exist(&bin_path) && is_executable(&bin_path) {
                return Some(bin_path);
            }
        }
        return None;
    });
    match path_buf {
        Some(path) => Ok(path),
        None => Err("cannot find the path"),
    }
}
// change binary name to OsStr
///if given an absolute path , returns it if file exists and is executable.
/// if given a relative path , returns an absolute path to the file if
/// it exists and is executable.

/// if given a string without path seprators, looks for a file named
/// 'binary_name' at each directory in '$PATH' and if it finds an executable
/// file there , returns it.
///
pub fn which<T: AsRef<OsStr>>(binary_name: T) -> Result<PathBuf, &'static str> {
    //which_in(binary_name, env::var_os("PATH"))
    env::current_dir()
        .or_else(|_| Err("Could n't get curent directory"))
        .and_then(|cwd| which_in(binary_name, env::var_os("PATH"), &cwd))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    struct TestFixture {
        ///Temp directory
        pub tempdir: TempDir,
        ///$Path
        pub paths: OsString,
        ///Binaries created in Path
        pub bins: Vec<PathBuf>,
    }
    const SUBDIRS: &'static [&'static str] = &["a", "b", "c"];
    const BIN_NAME: &'static str = "bin";

    #[cfg(unix)]
    fn mk_bin(dir: &Path, path: &str) -> io::Result<PathBuf> {
        // use libc;
        use std::os::unix::fs::OpenOptionsExt;
        let bin = dir.join(path);
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o666 | (libc::S_IXUSR as u32))
            .open(&bin)
            .and_then(|_f| bin.canonicalize())
    }

    fn touch(dir: &Path, path: &str) -> io::Result<PathBuf> {
        let b = dir.join(path);
        fs::File::create(&b).and_then(|_f| b.canonicalize())
    }

    #[cfg(not(unix))]
    fn mk_bin(dir: &Path, path: &str) -> io::Result<PathBuf> {
        touch(dir, path)
    }
    impl TestFixture {
        pub fn new() -> TestFixture {
            let tempdir = TempDir::new("which_tests").unwrap();
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true);
            let mut paths = vec![];
            let mut bins = vec![];
            for d in SUBDIRS.iter() {
                let p = tempdir.path().join(d);
                builder.create(&p).unwrap();
                bins.push(mk_bin(&p, &BIN_NAME).unwrap());
                paths.push(p);
            }
            TestFixture {
                tempdir: tempdir,
                paths: env::join_paths(paths).unwrap(),
                bins: bins,
            }
        }

        #[allow(dead_code)]
        pub fn touch(&self, path: &str) -> io::Result<PathBuf> {
            touch(self.tempdir.path(), &path)
        }

        pub fn mk_bin(&self, path: &str) -> io::Result<PathBuf> {
            mk_bin(self.tempdir.path(), &path)
        }
    }
    fn _which(f: &TestFixture, path: &str) -> Result<PathBuf, &'static str> {
        which_in(path, Some(f.paths.clone()))
    }

    #[test]
    fn test_which() {
        let f = TestFixture::new();
        assert_eq!(_which(&f, &BIN_NAME).unwrap(), f.bins[0])
    }
    #[test]
    fn test_which_not_found() {
        let f = TestFixture::new();
        assert!(_which(&f, "a").is_err());
    }
    #[test]
    fn test_which_second() {
        let f = TestFixture::new();
        let b = f.mk_bin("b/another").unwrap();
        assert_eq!(_which(&f, "another").unwrap(), b);
    }
    #[test]
    #[cfg(unix)]
    fn test_which_non_executable() {
        let f = TestFixture::new();
        f.touch("b/another").unwrap();
        assert!(_which(&f, "another").is_err());
    }
}

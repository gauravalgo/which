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
pub fn which_in<T, U,V>(binary_name: T, paths: Option<U>, cwd: V) -> Result<PathBuf, &'static str>
where
    T: AsRef<OsStr>,
    U: AsRef<OsStr>,
    V: AsRef<Path>
{
    //does it have a path seprator ?
    let path = Path::new(binary_name.as_ref());
    if path.components().count() > 1 {
        if path.is_absolute() {
            if is_exist(path) && is_executable(path){
                //already fine
                Ok(PathBuf::from(path))
            }else {
                //absolute path its not usable
                Err("bad absolute path")
            }
        }
        else {
            //try to make it absolute
            
            let mut new_path = PathBuf::from(cwd.as_ref());
            new_path.push(path);
            if is_exist(&new_path) && is_executable(&new_path){
                Ok(new_path)
            }
            else {
                //File does not exist or is not executable
                Err("Bad relative path")
            }
        }
    
    }
    else {
       //No seprator then look it up in paths
       paths.and_then(|paths| env::split_paths(paths.as_ref())
       .map(|p| p.join(binary_name.as_ref()))
       .skip_while(|p| !(is_exist(&p) && is_executable(&p)))
        .next())
        .ok_or("cannot find binary path")
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
    use std::ffi::{OsString,OsStr};
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
    fn _which<T: AsRef<OsStr>>(f: &TestFixture, path: T) -> Result<PathBuf, &'static str> {
        which_in(path, Some(f.paths.clone()),f.tempdir.path())
    }

    #[test]
    fn test_which() {
        let f = TestFixture::new();
        assert_eq!(_which(&f, &BIN_NAME).unwrap().canonicalize().unwrap(), f.bins[0])
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
        assert_eq!(_which(&f, "another").unwrap().canonicalize().unwrap(), b);
    }
    #[test]
    #[cfg(unix)]
    fn test_which_non_executable() {
        let f = TestFixture::new();
        f.touch("b/another").unwrap();
        assert!(_which(&f, "another").unwrap().canonicalize().is_err());
    }
    #[test]
    fn test_which_absolute(){
        let f = TestFixture::new();
        assert_eq!(_which(&f, &f.bins[1]).unwrap().canonicalize().unwrap(),f.bins[1].canonicalize().unwrap());
    }
    #[test]
    fn test_which_relative() {
        let f = TestFixture::new();
        assert_eq!(_which(&f, "b/bin").unwrap().canonicalize().unwrap(),f.bins[1].canonicalize().unwrap());
    }

    #[test]
    fn test_with_relative_leading_dot(){
        let f = TestFixture::new();
        assert_eq!(_which(&f, "./b/bin").unwrap().canonicalize().unwrap(),f.bins[1].canonicalize().unwrap());

    }
    #[test]
    #[cfg(unix)]
    fn test_which_absolute_non_executable(){
        //should not return non-executable files, even if given an absolute path.
        let f = TestFixture::new();
        let b = f.touch("b/another").unwrap();
        assert!(_which(&f, &b).is_err());
    }
    #[test]
    #[cfg(unix)]
    fn test_which_relative_non_executable(){
    //should not return non - executable files
    let f = TestFixture::new();
    f.touch("b/another").unwrap();
    assert!(_which(&f, "b/another").is_err());
    }
    
}

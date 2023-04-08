
extern crate libc;

use std::path::{Path,PathBuf};
use std::{env, fs};
#[cfg(unix)]
use std::ffi::CString;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

pub fn is_exist(binpath: &PathBuf) ->bool {
    match fs::metadata(binpath).map(|metadata|{
        metadata.is_file()
    })  {
        Ok(true) => true,
        _ => false,
    }
}  
//return true is path is executable
#[cfg(unix)]
fn is_executable(path: &Path)-> bool{
    CString::new(path.as_os_str().as_bytes()).
        and_then(|c| {
            Ok(unsafe{ libc::access(c.as_ptr(),libc::X_OK) == 0})
                })
                .unwrap_or(false)

}
#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool{
    true
}
// change binary name to OsStr
pub fn which<T: AsRef<OsStr>>(binary_name : T) -> Result<PathBuf,&'static str> 
{
    let path_buf = env::var_os("PATH").and_then(
        |paths| -> Option<PathBuf>{
            for path in env::split_paths(&paths) {
                let bin_path = path.join(binary_name.as_ref()); 
                if is_exist(&bin_path) && is_executable(&bin_path){
                    return Some(bin_path);
                }
            }
        return None;
        });
    match path_buf {
        Some(path) => Ok(path),
        None => Err("cannot find the path")
    }
}
#[test]
fn it_works() {
    use std::process::Command;      
    let result = which("rustc");
    assert!(result.is_ok()); 

    let which_result = Command::new("which")
        .arg("rustc")
        .output();

    assert_eq!(String::from(result.unwrap().to_str().unwrap()),
            String::from_utf8(which_result.unwrap().stdout).unwrap().trim());


}
#[test]
fn do_it_works() {
    let result = which("cargo does not exist");
    assert_eq!(result,Err("cannot find the path") );
}



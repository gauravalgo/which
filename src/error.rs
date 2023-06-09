use std::fmt;

pub type Result<T> = std::result::Result<T,Error>;

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum Error {
    BadAbsolutePath,
    BadRelativePath,
    CannotFindBinaryPath,
    CannotGetCurrentDir,
    CannotCannonicalize,
}
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self,f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadAbsolutePath => write!(f,"bad absolute path"),
            Error::BadRelativePath => write!(f,"bad relative path"),
            Error::CannotFindBinaryPath => write!(f,"cannot find binary path"),
            Error::CannotGetCurrentDir => write!(f,"cannot get current directory"),
            Error::CannotCannonicalize => wrie!(f,"cannot cannonicalize path"),
        }
    }

}
#which 
A Rust equvilent of unix command "which".

##Example
To find which rustc executable binary is using.
```rust
using which::which;

let result = which::which("rustc").unwarp();
assert_eq!(result,PathBuf::from("/usr/bin/rustc"));



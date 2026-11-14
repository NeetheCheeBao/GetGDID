use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    let res = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("app.res");
    if compile_with_rc(&res) || compile_with_windres(&res) {
        println!("cargo:rustc-link-arg={}", res.display());
    } else {
        println!("cargo:warning=rc.exe 与 windres 均不可用，可执行文件将不含图标资源");
    }
}

fn compile_with_rc(out: &Path) -> bool {
    Command::new("rc.exe")
        .args(["/nologo", "/fo"])
        .arg(out)
        .arg("app.rc")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn compile_with_windres(out: &Path) -> bool {
    Command::new("windres")
        .args(["--target=pe-x86-64", "-O", "res", "-i", "app.rc", "-o"])
        .arg(out)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

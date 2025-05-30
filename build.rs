use std::env;
use std::process::Command;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=dylib=mfplat");
        println!("cargo:rustc-link-lib=dylib=strmiids");
        println!("cargo:rustc-link-lib=dylib=mfuuid");

        if env::var("VCINSTALLDIR").is_err() {
            if let Some(vs_path) = find_vs_path() {
                setup_vc_environment(&vs_path);
            }
        }
    }
}

fn setup_vc_environment(vs_path: &str) {
    let vcvars = format!(r"{}\VC\Auxiliary\Build\vcvars64.bat", vs_path);
    if std::path::Path::new(&vcvars).exists() {
        println!("cargo:warning=Setting up VC environment using {}", vcvars);
        let output = Command::new("cmd")
            .args(["/C", &vcvars, "&&", "set"])
            .output()
            .expect("Failed to execute vcvars");

        if output.status.success() {
            let env_vars = String::from_utf8_lossy(&output.stdout);
            for line in env_vars.lines() {
                if let Some((var, value)) = line.split_once('=') {
                    unsafe { env::set_var(var, value) };
                }
            }
        }
    }
}

fn find_vs_path() -> Option<String> {
    let versions = ["2022", "2019", "2017"];
    let editions = ["Enterprise", "Professional", "Community", "BuildTools"];

    for version in versions {
        for edition in editions {
            let path = format!(
                r"C:\Program Files (x86)\Microsoft Visual Studio\{}\{}",
                version, edition
            );
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
    }
    None
}

use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../assets/brand/icons/tourne-la-page.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sdk = PathBuf::from(env::var_os("ProgramFiles(x86)").expect("Windows SDK location"))
        .join("Windows Kits/10/bin");
    let mut compilers = fs::read_dir(sdk)
        .expect("Windows SDK must be installed")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("x64/rc.exe"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    compilers.sort();
    let compiler = compilers
        .last()
        .expect("Windows resource compiler must be installed");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../assets/brand/icons/tourne-la-page.ico")
        .canonicalize()
        .expect("application icon");
    let resource = out.join("tourne-la-page.rc");
    fs::write(
        &resource,
        format!(
            "#pragma code_page(65001)\n1 ICON \"{}\"\n",
            icon.display().to_string().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    let compiled = out.join("tourne-la-page.res");
    let result = Command::new(compiler)
        .args(["/nologo", "/fo"])
        .arg(&compiled)
        .arg(&resource)
        .status()
        .expect("resource compiler");
    assert!(result.success(), "application icon compilation failed");
    println!("cargo:rustc-link-arg={}", compiled.display());
}

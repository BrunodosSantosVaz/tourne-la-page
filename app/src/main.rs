#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod brand;
mod fonts;
mod library;
mod reader;
mod runtime;

#[cfg(test)]
mod brand_tests;
#[cfg(test)]
mod fonts_tests;
#[cfg(test)]
mod runtime_tests;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("app crate must be inside the workspace")?;
    let data_dir = runtime::RuntimeConfig::from_environment(root)?.prepare()?;
    let mut arguments = std::env::args().skip(1);
    if arguments.next().as_deref() == Some("--verify-install") {
        let _store = storage::LibraryStore::open(&data_dir)?;
        let report = serde_json::to_string(&serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"), "data_directory": data_dir, "database": "ok"
        }))?;
        if let Some(destination) = arguments.next() {
            std::fs::write(destination, report)?;
        } else {
            println!("{report}");
        }
        return Ok(());
    }
    reader::run(data_dir)?;
    Ok(())
}

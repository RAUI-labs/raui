use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let mut output = String::new();
    output.push_str("#![allow(ambiguous_glob_reexports)]\n");
    output.push_str("#![allow(unused_variables)]\n");

    visit_dirs(
        Path::new("../core/src"),
        "raui_core",
        None,
        &mut output,
        &[],
    );
    visit_dirs(
        Path::new("../material/src"),
        "raui_material",
        Some("material"),
        &mut output,
        &[],
    );
    visit_dirs(
        Path::new("../retained/src"),
        "raui_retained",
        Some("retained"),
        &mut output,
        &[],
    );
    visit_dirs(
        Path::new("../immediate/src"),
        "raui_immediate",
        Some("immediate"),
        &mut output,
        &[],
    );
    visit_dirs(
        Path::new("../immediate-widgets/src"),
        "raui_immediate_widgets",
        Some("immediate-widgets"),
        &mut output,
        &[],
    );
    visit_dirs(
        Path::new("../tesselate-renderer/src"),
        "raui_tesselate_renderer",
        Some("tesselate"),
        &mut output,
        &[],
    );
    visit_dirs(
        Path::new("../json-renderer/src"),
        "raui_json_renderer",
        Some("json"),
        &mut output,
        &[],
    );
    visit_dirs(
        Path::new("../app/src"),
        "raui_app",
        Some("app"),
        &mut output,
        &[
            "asset_manager.rs",
            "interactions.rs",
            "text_measurements.rs",
        ],
    );

    let out_path = Path::new("src").join("import_all.rs");
    let mut file = File::create(&out_path).expect("Failed to create import_all.rs");
    file.write_all(output.as_bytes()).expect("Write failed");
}

fn visit_dirs(
    dir: &Path,
    prefix: &str,
    feature: Option<&str>,
    output: &mut String,
    ignore: &[&str],
) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            if path.join("mod.rs").exists() {
                let mod_path = path.strip_prefix(dir).unwrap();
                let mod_name = mod_path.to_string_lossy().replace("/", "::");
                if let Some(feature) = feature {
                    output.push_str(&format!("#[cfg(feature = \"{feature}\")]\n"));
                }
                output.push_str(&format!("pub use {prefix}::{mod_name}::*;\n"));
                visit_dirs(
                    &path,
                    &format!("{prefix}::{mod_name}"),
                    feature,
                    output,
                    ignore,
                );
            }
        } else if let Some(ext) = path.extension() {
            if ext == "rs" {
                if path.file_name().unwrap() == "lib.rs" {
                    if let Some(feature) = feature {
                        output.push_str(&format!("#[cfg(feature = \"{feature}\")]\n"));
                    }
                    output.push_str(&format!("pub use {prefix}::*;\n"));
                    continue;
                }

                if path.file_name().unwrap() == "mod.rs"
                    || path.file_name().unwrap() == "import_all.rs"
                    || ignore.iter().any(|name| path.file_name().unwrap() == *name)
                {
                    continue;
                }

                let mod_path = path.strip_prefix(dir).unwrap();
                let mut mod_name = mod_path.to_string_lossy().replace("/", "::");
                mod_name = mod_name.trim_end_matches(".rs").to_string();
                if let Some(feature) = feature {
                    output.push_str(&format!("#[cfg(feature = \"{feature}\")]\n"));
                }
                output.push_str(&format!("pub use {prefix}::{mod_name}::*;\n"));
            }
        }
    }
}

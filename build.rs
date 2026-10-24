//! Generates one libtest function for every scenario suite case.

#![expect(
    clippy::expect_used,
    reason = "Cargo inputs and generated files are required to build the test targets"
)]

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR");
    let ron_dir = Path::new(&manifest).join("tests/ron");
    let mut files: Vec<_> = fs::read_dir(&ron_dir)
        .expect("read tests/ron")
        .map(|entry| entry.expect("read scenario directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ron"))
        .filter(|path| {
            !path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".scene.ron"))
        })
        .collect();
    files.sort();

    println!("cargo:rerun-if-changed=tests/ron");
    println!("cargo:rerun-if-changed=tests/scenarios.rs");
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("Cargo sets OUT_DIR"));

    let mut ron_tests = String::new();
    let mut image_tests = String::new();
    let mut shuffled_tests = String::new();
    let variants = ["default", "scale_2", "text_1_5x"];
    let schedules = ["PreUpdate", "Update", "PostUpdate", "Last"];
    let seeds = [1_u64, 2, 3, 0x5eed_cafe];

    for path in files {
        let file = path.file_name().expect("RON file name").to_string_lossy();
        let stem = path.file_stem().expect("RON file stem").to_string_lossy();
        let disabled = fs::read_to_string(&path)
            .expect("read RON scenario")
            .lines()
            .any(|line| line.starts_with("    skip:") || line.starts_with("    fixme:"));
        let ignore = if disabled {
            "#[ignore = \"scenario is marked skip or fixme\"]\n"
        } else {
            ""
        };
        let path = format!("concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/tests/ron/{file}\")");
        for variant in variants {
            let test_name = format!("scenario_{}_{}", rust_name(&stem), variant);
            let _ = writeln!(
                ron_tests,
                "{ignore}#[test]\nfn {test_name}() {{ run_ron_case({path}, {variant:?}); }}"
            );
        }
        let image_name = format!("scenario_{}_drawn_to_image", rust_name(&stem));
        let _ = writeln!(
            image_tests,
            "{ignore}#[test]\nfn {image_name}() {{ run_image_case({path}); }}"
        );
        for schedule in schedules {
            for seed in seeds {
                let test_name = format!(
                    "scenario_{}_{}_seed_{:x}",
                    rust_name(&stem),
                    schedule.to_lowercase(),
                    seed
                );
                let seed = if seed == 0x5eed_cafe {
                    "0x5eed_cafe".to_owned()
                } else {
                    seed.to_string()
                };
                let _ = writeln!(
                    shuffled_tests,
                    "{ignore}#[test]\nfn {test_name}() {{ run_shuffled_case({path}, {schedule:?}, {seed}); }}"
                );
            }
        }
    }

    fs::write(output.join("ron_scenario_tests.rs"), ron_tests).expect("write generated RON tests");
    fs::write(output.join("image_scenario_tests.rs"), image_tests)
        .expect("write generated image tests");
    fs::write(output.join("shuffled_scenario_tests.rs"), shuffled_tests)
        .expect("write generated shuffled tests");
}

fn rust_name(name: &str) -> String {
    let mut result = String::new();
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character);
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    result.trim_matches('_').to_owned()
}

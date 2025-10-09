use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct Recompiler {
    config_path: PathBuf,
    cache_dir: PathBuf,
    binary_path: PathBuf,
}

impl Recompiler {
    pub fn new() -> Result<Self> {
        let home = env::var("HOME").context("HOME not set")?;
        let config_path = PathBuf::from(&home).join(".config/oxwm/config.rs");
        let cache_dir = PathBuf::from(&home).join(".cache/oxwm");
        let binary_path = cache_dir.join("oxwm");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            config_path,
            cache_dir,
            binary_path,
        })
    }

    pub fn recompile(&self) -> Result<CompileResult> {
        if !self.config_path.exists() {
            return Ok(CompileResult::NoConfig);
        }

        println!("Recompiling oxwm...");

        let build_dir = self.cache_dir.join("build");
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
        }
        fs::create_dir_all(&build_dir)?;

        self.setup_build_project(&build_dir)?;

        let output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&build_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            let compiled = build_dir.join("target/release/oxwm");
            fs::copy(&compiled, &self.binary_path)?;

            println!("Compilation successful!");
            Ok(CompileResult::Success)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("\n{}", "=".repeat(60));
            eprintln!("OXWM COMPILATION FAILED");
            eprintln!("{}", "=".repeat(60));
            eprintln!("{}", stderr);
            eprintln!("{}", "=".repeat(60));

            let _ = self.show_error_notification(&stderr);

            Ok(CompileResult::Error(stderr.to_string()))
        }
    }

    fn setup_build_project(&self, build_dir: &PathBuf) -> Result<()> {
        let oxwm_src = if PathBuf::from("/usr/share/oxwm/src").exists() {
            PathBuf::from("/usr/share/oxwm")
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        };

        let cargo_toml_path = oxwm_src.join("Cargo.toml");
        if cargo_toml_path.exists() {
            fs::copy(&cargo_toml_path, build_dir.join("Cargo.toml"))?;
        } else {
            let cargo_toml = r#"
[package]
name = "oxwm"
version = "0.1.0"
edition = "2024"

[dependencies]
x11 = { version = "2.21", features = ["xlib", "xft"] }
x11rb = "0.13"
anyhow = "1"
chrono = "0.4"

[profile.release]
opt-level = 3
"#;
            fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;
        }

        let src_dir = oxwm_src.join("src");
        let build_src = build_dir.join("src");
        copy_dir_all(&src_dir, &build_src)?;

        fs::copy(&self.config_path, build_src.join("config.rs"))?;

        let default_config_src = oxwm_src.join("default_config.rs");
        if default_config_src.exists() {
            fs::copy(&default_config_src, build_dir.join("default_config.rs"))?;
        }

        fs::write(build_dir.join("build.rs"), "fn main() {}")?;

        Ok(())
    }

    fn show_error_notification(&self, error: &str) -> Result<()> {
        let summary = "OXWM Compilation Failed";
        let body = error.lines().take(5).collect::<Vec<_>>().join("\n");

        let _ = Command::new("notify-send")
            .args(&["-u", "critical", "-t", "10000", summary, &body])
            .spawn();

        Ok(())
    }

    pub fn get_binary_path(&self) -> &PathBuf {
        &self.binary_path
    }
}

pub enum CompileResult {
    Success,
    Error(String),
    NoConfig,
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

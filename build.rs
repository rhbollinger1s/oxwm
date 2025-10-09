use std::env;
use std::path::PathBuf;

fn main() {
    let config_path = find_config();
    println!("cargo:rustc-env=OXWM_CONFIG_PATH={}", config_path.display());
    println!("cargo:rerun-if-changed={}", config_path.display());

    if config_path.to_str().unwrap().contains(".config/oxwm") {
        println!("cargo:warning=Using user config: {}", config_path.display());
    }
}

fn find_config() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        let user_config = PathBuf::from(home).join(".config/oxwm/config.rs");
        if user_config.exists() {
            return user_config;
        }
    }

    let system_config = PathBuf::from("/usr/share/oxwm/config.rs");
    if system_config.exists() {
        return system_config;
    }

    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("default_config.rs")
}

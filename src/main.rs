use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
mod bar;
mod config;
mod keyboard;
mod layout;
mod recompile;
mod window_manager;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "init" => {
                init_user_config()?;
                return Ok(());
            }
            "recompile" => {
                let recompiler = recompile::Recompiler::new()?;
                match recompiler.recompile()? {
                    recompile::CompileResult::Success => {
                        println!("Successfully compiled!");
                    }
                    recompile::CompileResult::Error(e) => {
                        eprintln!("Compilation failed:\n{}", e);
                        std::process::exit(1);
                    }
                    recompile::CompileResult::NoConfig => {
                        eprintln!("No config found. Run 'oxwm init' first.");
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }
            _ => {}
        }
    }

    let mut window_manager = window_manager::WindowManager::new()?;
    let should_restart = window_manager.run()?;

    drop(window_manager);

    if should_restart {
        let recompiler = recompile::Recompiler::new()?;
        let recompiled_binary = recompiler.get_binary_path();

        let binary_to_exec = if recompiled_binary.exists() {
            println!(
                "Restarting with recompiled binary: {}",
                recompiled_binary.display()
            );
            recompiled_binary.to_str().unwrap()
        } else {
            &args[0]
        };

        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(binary_to_exec)
            .args(&args[1..])
            .exec();
        eprintln!("Failed to restart: {}", err);
    }

    Ok(())
}

fn init_user_config() -> Result<()> {
    let home = env::var("HOME")?;
    let config_dir = PathBuf::from(&home).join(".config/oxwm");
    let config_file = config_dir.join("config.rs");

    fs::create_dir_all(&config_dir)?;

    if config_file.exists() {
        println!("Config already exists at {}", config_file.display());
        return Ok(());
    }

    let default_config = include_str!("../default_config.rs");
    fs::write(&config_file, default_config)?;

    println!("Created config at {}", config_file.display());
    println!("\nEdit your config:");
    println!("  $EDITOR {}", config_file.display());
    println!("\nThen recompile and restart:");
    println!("  Press Super+Shift+R in oxwm");
    println!("  Or run: oxwm recompile && oxwm");

    Ok(())
}

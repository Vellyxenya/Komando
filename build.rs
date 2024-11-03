// use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Run pre-install setup
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
        // Create necessary directories
        let home_dir = env::var("HOME").unwrap();
        let config_dir = Path::new(&home_dir).join(".config").join("komando");
        
        std::fs::create_dir_all(&config_dir).unwrap_or_else(|err| {
            println!("cargo:warning=Failed to create config directory: {}", err);
        });
    }
    
    println!("cargo:warning=Pre-install setup completed");
}
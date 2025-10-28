use std::process::Command;
use std::path::Path;

fn main() {
    // 1. Initialize the database if it doesn't exist.
    println!("cargo:rerun-if-changed=init.sql");
    if !Path::new("solve_np.db3").exists() {
        let output = Command::new("sh")
            .arg("-c")
            .arg("sqlite3 solve_np.db3 < init.sql")
            .output()
            .expect("Failed to execute sqlite3 command");

        if !output.status.success() {
            panic!(
                "Failed to initialize database: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // 2. Build the JS subproject.
    println!("cargo:rerun-if-changed=js/package.json");
    println!("cargo:rerun-if-changed=js/package-lock.json");
    println!("cargo:rerun-if-changed=js/tsconfig.json");
    println!("cargo:rerun-if-changed=js/src");
    let esbuild_status = Command::new("npx")
        .arg("esbuild")
        .arg("js/src/index.ts")
        .arg("--outfile=js/dist/bundle.js")
        .arg("--bundle")
        .status()
        .expect("Failed to run esbuild");

    if !esbuild_status.success() {
        panic!("esbuild failed");
    }
}

use std::fs;
use std::path::Path;

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn sync_dist() {
    let root = Path::new("..");
    let dist = Path::new("../dist");
    let _ = fs::create_dir_all(dist);

    if root.join("index.html").exists() {
        let _ = fs::copy(root.join("index.html"), dist.join("index.html"));
    }
    if root.join("js").exists() {
        let _ = copy_dir_all(root.join("js"), dist.join("js"));
    }
    if root.join("css").exists() {
        let _ = copy_dir_all(root.join("css"), dist.join("css"));
    }
    if root.join("assets").exists() {
        let _ = copy_dir_all(root.join("assets"), dist.join("assets"));
    }
}

fn main() {
    sync_dist();
    println!("cargo:rerun-if-changed=../js");
    println!("cargo:rerun-if-changed=../css");
    println!("cargo:rerun-if-changed=../index.html");
    println!("cargo:rerun-if-changed=../assets");
    tauri_build::build();
}

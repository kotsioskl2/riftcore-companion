use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let champions_dir = manifest_dir.join("assets").join("champions");
    let logo_path = manifest_dir
        .parent()
        .unwrap()
        .join("public")
        .join("logo.png");
    let mut portrait_paths: Vec<PathBuf> = fs::read_dir(&champions_dir)
        .expect("champion portrait assets are required")
        .map(|entry| entry.expect("failed to read portrait asset entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    portrait_paths.sort();

    let mut embedded = String::from("pub static CHAMPION_PORTRAITS: &[(&str, &[u8])] = &[\n");
    for path in &portrait_paths {
        let key = path.file_stem().unwrap().to_string_lossy();
        embedded.push_str(&format!(
            "    ({key:?}, include_bytes!({:?}) as &[u8]),\n",
            path.to_string_lossy()
        ));
        println!("cargo:rerun-if-changed={}", path.display());
    }
    embedded.push_str("];\n");
    embedded.push_str(&format!(
        "pub static RIFTCORE_LOGO: &[u8] = include_bytes!({:?});\n",
        logo_path.to_string_lossy()
    ));
    let tft_icons_dir = manifest_dir.join("assets").join("tft_icons");
    let mut tft_icon_paths: Vec<PathBuf> = fs::read_dir(&tft_icons_dir)
        .expect("TFT icon assets are required; run scripts/download_tft_icons.mjs")
        .map(|entry| entry.expect("failed to read TFT icon asset entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    tft_icon_paths.sort();
    embedded.push_str("pub static TFT_ICONS: &[(&str, &[u8])] = &[\n");
    for path in &tft_icon_paths {
        let key = path.file_stem().unwrap().to_string_lossy();
        embedded.push_str(&format!(
            "    ({key:?}, include_bytes!({:?}) as &[u8]),\n",
            path.to_string_lossy()
        ));
        println!("cargo:rerun-if-changed={}", path.display());
    }
    embedded.push_str("];\n");
    fs::write(out_dir.join("embedded_assets.rs"), embedded)
        .expect("failed to generate embedded asset table");
    println!("cargo:rerun-if-changed={}", logo_path.display());
    println!("cargo:rerun-if-changed={}", champions_dir.display());
    println!("cargo:rerun-if-changed={}", tft_icons_dir.display());

    #[cfg(windows)]
    {
        let icon_src = manifest_dir.join("icon.ico");
        let icon_dest = out_dir.join("icon.ico");

        println!("cargo:rerun-if-changed=icon.ico");
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

        if icon_src.exists() {
            let _ = fs::copy(&icon_src, &icon_dest);
        }

        let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
        let numeric_version = version.split(['-', '+']).next().unwrap_or("0.0.0");
        let mut version_parts = numeric_version
            .split('.')
            .map(|part| part.parse::<u16>().unwrap_or(0));
        let major = version_parts.next().unwrap_or(0);
        let minor = version_parts.next().unwrap_or(0);
        let patch = version_parts.next().unwrap_or(0);
        let fixed_version = format!("{major}, {minor}, {patch}, 0");

        let rc_content = r#"#pragma code_page(65001)
1 VERSIONINFO
FILEVERSION 1, 0, 0, 0
PRODUCTVERSION 1, 0, 0, 0
FILEFLAGSMASK 0x3f
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE 0x1
FILESUBTYPE 0x0
{
BLOCK "StringFileInfo"
{
BLOCK "000004b0"
{
VALUE "ProductName", "RiftCore Companion"
VALUE "FileDescription", "RiftCore League of Legends Companion"
VALUE "ProductVersion", "1.1.0"
VALUE "FileVersion", "1.1.0"
VALUE "LegalCopyright", "Copyright (c) 2026 kotsioskl2"
}
}
BLOCK "VarFileInfo" {
VALUE "Translation", 0x0, 0x04b0
}
}
1 ICON "icon.ico"
"#
        .replace(
            "FILEVERSION 1, 0, 0, 0",
            &format!("FILEVERSION {fixed_version}"),
        )
        .replace(
            "PRODUCTVERSION 1, 0, 0, 0",
            &format!("PRODUCTVERSION {fixed_version}"),
        )
        .replace("\"1.1.0\"", &format!("\"{version}\""));

        let rc_path = out_dir.join("resource.rc");
        fs::write(&rc_path, rc_content).expect("Failed to write resource.rc");

        let resource_o = out_dir.join("resource.o");

        // Locate windres (PATH or w64devkit)
        let windres_candidates = ["windres", "x86_64-w64-mingw32-windres"];

        let mut compiled = false;
        for cmd in &windres_candidates {
            let status = Command::new(cmd)
                .current_dir(&out_dir)
                .args(["-i", "resource.rc", "-o", "resource.o", "-O", "coff"])
                .status();

            if let Ok(s) = status {
                if s.success() {
                    compiled = true;
                    break;
                }
            }
        }

        if compiled && resource_o.exists() {
            println!("cargo:rustc-link-arg={}", resource_o.display());
        } else {
            eprintln!("Warning: could not compile resource.o using windres");
        }
    }
}

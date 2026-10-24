use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    #[cfg(windows)]
    {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let icon_src = manifest_dir.join("icon.ico");
        let icon_dest = out_dir.join("icon.ico");

        println!("cargo:rerun-if-changed=icon.ico");
        println!("cargo:rerun-if-changed=build.rs");

        if icon_src.exists() {
            let _ = fs::copy(&icon_src, &icon_dest);
        }

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
"#;

        let rc_path = out_dir.join("resource.rc");
        fs::write(&rc_path, rc_content).expect("Failed to write resource.rc");

        let resource_o = out_dir.join("resource.o");

        // Locate windres (PATH or w64devkit)
        let windres_candidates = [
            "windres",
            "x86_64-w64-mingw32-windres",
            r"F:\w64devkit\bin\windres.exe",
        ];

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

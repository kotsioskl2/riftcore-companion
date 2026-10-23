fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("ProductName", "RiftCore Companion");
        res.set("FileDescription", "RiftCore League of Legends Companion");
        res.set("LegalCopyright", "Copyright (c) 2026 kotsioskl2");
        if let Err(e) = res.compile() {
            eprintln!("Warning: failed to compile Windows resources: {}", e);
        }
    }
}

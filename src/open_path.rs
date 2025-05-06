pub fn open_path(path: &str) {
    // Open the temporary directory in the file explorer
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .expect("Failed to open temporary directory");
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .expect("Failed to open temporary directory");
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn()
            .expect("Failed to open temporary directory");
    }
}

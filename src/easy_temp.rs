use std::{fs, path::PathBuf};
use uuid::Uuid;

pub struct EasyTemp {
    pub path: PathBuf,
}

impl EasyTemp {
    pub fn new() -> Self {
        let path = {
            let temp_dir = std::env::temp_dir();

            loop {
                // Append a UUIDv7 string to the temporary directory path
                let apk_temp_dir = temp_dir.join(Uuid::now_v7().to_string());

                if !apk_temp_dir.exists() {
                    // Create the directory if it doesn't exist
                    fs::create_dir_all(&apk_temp_dir).expect("Failed to create temporary directory");
                    break apk_temp_dir;
                }
            }
        };
        Self { path }
    }
}

impl Drop for EasyTemp {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).expect("Failed to clean up temporary directory");
        }
    }
}

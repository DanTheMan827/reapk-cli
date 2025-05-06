use std::{env, fs};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use uuid::Uuid;

pub fn run_java_jar(jar_path: &str, args: &[&str]) -> Result<ExitStatus, ExitStatus> {
    // Locate the Java executable
    let java_path = locate_executable("java").expect("Java executable not found");

    // Build the command to run `java -jar`
    let mut child = Command::new(java_path)
        .arg("-jar")
        .arg(jar_path)
        .args(args) // Add additional arguments
        .stdout(Stdio::inherit()) // Redirect stdout
        .stderr(Stdio::inherit()) // Redirect stderr
        .spawn()
        .expect("Failed to spawn process"); // Spawn the process

    let exit_status = child.wait().expect("Failed to wait for process"); // Wait for the process to finish

    if exit_status.success() {
        Ok(exit_status)
    } else {
        Err(exit_status)
    }
}

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

/// Attempts to locate an executable by checking direct paths, the PATH variable,
/// and the App Paths registry on Windows.
pub fn locate_executable(command: &str) -> Option<String> {
    // Step 1: If the command is a direct path, check if it exists.
    let command_path = Path::new(command);
    if command_path.exists() && command_path.is_file() {
        return Some(command_path.to_string_lossy().to_string());
    }

    // If the command doesn't have an extension, consider adding ".exe"
    let candidate_names: Vec<String> = if Path::new(command).extension().is_none() {
        vec![command.to_string(), format!("{}.exe", command)]
    } else {
        vec![command.to_string()]
    };

    // Step 2: Look through each directory in the PATH environment variable.
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            for candidate in &candidate_names {
                let full_path = path.join(candidate);
                if full_path.exists() && full_path.is_file() {
                    return Some(full_path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Step 3: Check the App Paths registry (Windows-specific)
    #[cfg(windows)]
    {
        use std::path::PathBuf;
        use winreg::enums::*;
        use winreg::RegKey;

        // Open the registry key for App Paths.
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(app_paths) =
            hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths")
        {
            for candidate in &candidate_names {
                if let Ok(subkey) = app_paths.open_subkey(candidate) {
                    // The default value usually contains the full path to the executable.
                    if let Ok(path_str) = subkey.get_value::<String, _>("") {
                        let candidate_path = PathBuf::from(path_str);
                        if candidate_path.exists() && candidate_path.is_file() {
                            return Some(candidate_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    // No method succeeded, so return None.
    None
}

pub fn make_temp_dir() -> Result<PathBuf, std::io::Error> {
    // Store the temporary directory path
    let temp_dir = std::env::temp_dir();

    // Loop until we find a unique directory name not already in use
    loop {
        // Append a UUIDv7 string to the temporary directory path
        let temp_dir = temp_dir.join(Uuid::now_v7().to_string());

        if !temp_dir.exists() {
            // Create the directory if it doesn't exist
            let creation_result = fs::create_dir_all(&temp_dir);

            if creation_result.is_err() {
                // Return an error if the directory creation fails
                break Err(creation_result.unwrap_err());
            } else {
                // Return the path to the created directory
                break Ok(temp_dir);
            }

        }
    }
}

use std::env;
use std::path::Path;

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

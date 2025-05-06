use std::{env, fs};
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::process::exit;

fn main() {
    let out_path = Path::new(".").join("apktool.jar");

    if out_path.exists() {
        println!("apktool.jar already exists, skipping download.");

        return;
    }

    // Create a client with custom user agent to avoid GitHub API 403 errors
    let client = reqwest::blocking::Client::builder()
        .user_agent("reapk-cli-build-script/1.0")
        .build()
        .expect("Failed to create HTTP client");

    // Get the latest releases
    let response = client
        .get("https://api.github.com/repos/iBotPeaches/Apktool/releases/latest")
        .send()
        .expect("Failed to fetch latest release metadata");

    if !response.status().is_success() {
        eprintln!("GitHub API returned error: {}", response.status());
        exit(1);
    }

    let release: serde_json::Value = response.json().expect("Failed to parse JSON");
    let assets = release["assets"].as_array().expect("Invalid assets format");
    let jar_asset = assets.iter().find(|asset| {
        asset["name"]
            .as_str()
            .map(|name| name.starts_with("apktool_") && name.ends_with(".jar"))
            .unwrap_or(false)
    });

    let jar_url = match jar_asset.and_then(|a| a["browser_download_url"].as_str()) {
        Some(url) => url,
        None => {
            eprintln!("Could not find apktool jar in the latest release.");
            exit(1);
        }
    };

    let mut response = reqwest::blocking::get(jar_url)
        .expect("Failed to download apktool jar");

    println!("Downloading apktool jar from {} to {}", jar_url, out_path.to_string_lossy());

    let mut out_file = File::create(&out_path).expect("Failed to create output file");
    copy(&mut response, &mut out_file).expect("Failed to write apktool jar to disk");
}

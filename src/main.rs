mod easy_temp;
mod locate_executable;
mod open_path;

use std::{path::PathBuf, process::{Command, ExitStatus, Stdio}};
use clap::Parser;
use easy_temp::EasyTemp;
use locate_executable::locate_executable;
use open_path::open_path;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The APK file to be processed
    #[arg(index = 1)]
    input_apk: PathBuf,

    /// The APK file to be generated
    #[arg(index = 2)]
    output_apk: Option<PathBuf>,
}

fn main() {
    let debug_cert = include_bytes!("../debug_cert.crt");
    let debug_key = include_bytes!("../debug_key.pk8");
    let apk_signer = include_bytes!("../apksigner.jar");
    let apk_tool = include_bytes!("../apktool.jar");
    let (input_apk, output_apk) = {
        let args = Args::parse();
        let input_apk = args.input_apk;

        // Check if the input APK file exists
        if !input_apk.exists() {
            eprintln!("E: Input APK file does not exist.");
            std::process::exit(1);
        }

        if let Some(output_apk) = args.output_apk {
            (input_apk, output_apk)
        } else {
            (input_apk.clone(), input_apk.clone())
        }
    };



    // Determine a temporary directory for the unpacked APK
    let temp_dir = EasyTemp::new();
    let cert_path = temp_dir.path.join("debug_cert.crt");
    let key_path = temp_dir.path.join("debug_key.pk8");
    let apk_signer_path = temp_dir.path.join("apksigner.jar");
    let apk_tool_path = temp_dir.path.join("apktool.jar");
    let unpacked_path = temp_dir.path.join("unpacked");
    let intermediate_apk_path = temp_dir.path.join("intermediate.apk");

    // Write our cert and key files to the temporary directory
    std::fs::write(&cert_path, debug_cert).expect("Failed to write debug cert");
    std::fs::write(&key_path, debug_key).expect("Failed to write debug key");

    // Write the APK signer and APK tool JAR files to the temporary directory
    std::fs::write(&apk_signer_path, apk_signer).expect("Failed to write APK signer");
    std::fs::write(&apk_tool_path, apk_tool).expect("Failed to write APK tool");

    // Log our paths
    println!("I: Temporary directory: {}", temp_dir.path.to_string_lossy());
    println!("I: Input APK path: {}", input_apk.to_string_lossy());
    println!("I: Output APK path: {}", output_apk.to_string_lossy());

    // Unpack the APK using apktool
    println!("I: Unpacking input APK file to {}...", unpacked_path.to_string_lossy());
    let _ = run_java_jar(&apk_tool_path.to_string_lossy(), &[
        "d",
        &input_apk.to_string_lossy(),
        "-o",
        &unpacked_path.to_string_lossy(),
    ]).expect("Failed to unpack APK");

    // Open the temporary directory
    open_path(&unpacked_path.to_string_lossy());

    // Wait for the user to press Enter before proceeding
    println!("I: Press Enter to continue...");
    {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
    }

    // Pack the APK using apktool
    println!("I: Packing APK file to {}...", intermediate_apk_path.to_string_lossy());
    let _ = run_java_jar(&apk_tool_path.to_string_lossy(), &[
        "b",
        &unpacked_path.to_string_lossy(),
        "-o",
        &intermediate_apk_path.to_string_lossy(),
    ]).expect("Failed to unpack APK");

    // Clean up a little
    println!("I: Removing {}...", unpacked_path.to_string_lossy());
    std::fs::remove_dir_all(unpacked_path).expect("Failed to remove unpacked APK directory");
    println!("I: Removing {}...", apk_tool_path.to_string_lossy());
    std::fs::remove_file(apk_tool_path).expect("Failed to remove APK tool");

    // Sign the APK using apksigner
    println!("I: Signing APK file {}...", intermediate_apk_path.to_string_lossy());
    let _ = run_java_jar(&apk_signer_path.to_string_lossy(), &[
        "sign",
        "--key",
        &key_path.to_string_lossy(),
        "--cert",
        &cert_path.to_string_lossy(),
        &intermediate_apk_path.to_string_lossy(),
    ]).expect("Failed to unpack APK");

    // Copy the signed APK to the output path replacing any existing file
    println!("I: Copying signed APK to {}...", output_apk.to_string_lossy());
    std::fs::copy(&intermediate_apk_path, &output_apk)
        .expect("Failed to move signed APK to output path");

}

fn run_java_jar(jar_path: &str, args: &[&str]) -> Result<ExitStatus, ExitStatus> {
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

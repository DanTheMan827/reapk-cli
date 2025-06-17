mod utils;

use clap::Parser;
use defer_lite::defer;
use std::path::PathBuf;
use utils::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The APK file to be processed.
    #[arg(index = 1)]
    input_apk: PathBuf,

    /// The APK file to be generated.  If not provided, the input APK file will be used.
    #[arg(index = 2)]
    output_apk: Option<PathBuf>,

    /// The certificate file to use for signing the APK.  
    #[clap(long = "cert", requires = "key")]
    cert: Option<PathBuf>,

    /// The key file to use for signing the APK
    #[clap(long = "key", requires = "cert")]
    key: Option<PathBuf>,
}

fn main() {
    // Parse command line arguments
    let (input_apk, output_apk, cert, key) = {
        let args = Args::parse();
        let input_apk = args.input_apk;

        // Check if the input APK file exists
        if !input_apk.exists() {
            eprintln!("E: Input APK file does not exist.");
            std::process::exit(1);
        }

        // Check if the output APK argument is provided
        if let Some(output_apk) = args.output_apk {
            // Return the input APK and output APK paths
            (input_apk, output_apk, args.cert, args.key)
        } else {
            // The output APK is not provided, so we will just use the input APK path
            // as the output APK path
            (input_apk.clone(), input_apk.clone(), args.cert, args.key)
        }
    };

    // The bytes for our debug cert, key, apksigner, and apktool
    let debug_cert = include_bytes!("../debug_cert.crt");
    let debug_key = include_bytes!("../debug_key.pk8");
    let apk_signer = include_bytes!("../apksigner.jar");
    let apk_tool = include_bytes!("../apktool.jar");

    // Create a temporary directory
    let temp_dir = make_temp_dir().expect("Failed to create temporary directory");
    defer! {
        // Clean up the temporary directory when the scope ends
        println!("I: Cleaning up temporary directory...");
        std::fs::remove_dir_all(&temp_dir).expect("Failed to remove temporary directory");
    }

    // Define the paths for the temporary files
    let cert_path = &cert.clone().unwrap_or(temp_dir.join("debug_cert.crt"));
    let key_path = &key.clone().unwrap_or(temp_dir.join("debug_key.pk8"));
    let apk_signer_path = &temp_dir.join("apksigner.jar");
    let apk_tool_path = &temp_dir.join("apktool.jar");
    let unpacked_path = &temp_dir.join("unpacked");
    let intermediate_apk_path = &temp_dir.join("intermediate.apk");

    // Log our paths
    println!("I: Temporary directory: {}", temp_dir.to_string_lossy());
    println!("I: Input APK path: {}", input_apk.to_string_lossy());
    println!("I: Output APK path: {}", output_apk.to_string_lossy());
    println!("I: Unpacked APK path: {}", unpacked_path.to_string_lossy());

    // Scope that deals with unpacking and packing the APK
    {
        // Create a scope guard to clean up temporary files when the scope ends
        defer! {
            // Remove apktool if it exists
            if apk_tool_path.exists() {
                println!("I: Removing {}...", apk_tool_path.to_string_lossy());
                std::fs::remove_file(apk_tool_path).expect("Failed to remove APK tool");
            }

            // Remove the unpacked APK directory if it exists
            if unpacked_path.exists() {
                println!("I: Removing {}...", unpacked_path.to_string_lossy());
                std::fs::remove_dir_all(unpacked_path).expect("Failed to remove unpacked APK directory");
            }
        }

        // Write apktool to the temporary directory
        println!(
            "I: Writing APK tool to {}...",
            apk_tool_path.to_string_lossy()
        );
        std::fs::write(apk_tool_path, apk_tool).expect("Failed to write APK tool");

        // Unpack the APK using apktool
        println!(
            "I: Unpacking input APK file to {}...",
            &unpacked_path.to_string_lossy()
        );
        let _ = run_java_jar(
            apk_tool_path.to_string_lossy().as_ref(),
            &[
                "d",
                input_apk.to_string_lossy().as_ref(),
                "-o",
                unpacked_path.to_string_lossy().as_ref(),
            ],
        )
        .expect("Failed to unpack APK");

        // Open the temporary directory
        println!("I: Opening unpacked APK directory...");
        open_path(unpacked_path.to_string_lossy().as_ref());

        // Wait for the user to press Enter before proceeding
        println!("I: Press Enter to continue...");
        {
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
        }

        // Pack the APK using apktool
        println!(
            "I: Packing APK file to {}...",
            intermediate_apk_path.to_string_lossy()
        );
        let _ = run_java_jar(
            apk_tool_path.to_string_lossy().as_ref(),
            &[
                "b",
                unpacked_path.to_string_lossy().as_ref(),
                "-o",
                intermediate_apk_path.to_string_lossy().as_ref(),
            ],
        )
        .expect("Failed to pack APK");
    }

    // Scope that deals with signing the APK
    {
        // Create a scope guard to clean up temporary files when the scope ends
        defer! {
            // Remove apksigner if it exists
            if apk_signer_path.exists() {
                println!("I: Removing {}...", apk_signer_path.to_string_lossy());
                std::fs::remove_file(apk_signer_path).expect("Failed to remove APK signer");
            }

            // Remove debug cert if it exists
            if cert.is_none() && cert_path.exists() {
                println!("I: Removing {}...", cert_path.to_string_lossy());
                std::fs::remove_file(cert_path).expect("Failed to remove debug cert");
            }

            // Remove debug key if it exists
            if key.is_none() && key_path.exists() {
                println!("I: Removing {}...", key_path.to_string_lossy());
                std::fs::remove_file(key_path).expect("Failed to remove debug key");
            }
        }

        println!(
            "I: Writing APK signer to {}...",
            apk_signer_path.to_string_lossy()
        );
        
        if cert.is_none() {
            println!(
                "I: Writing debug certificate to {}...",
                cert_path.to_string_lossy()
            );
            std::fs::write(cert_path, debug_cert).expect("Failed to write debug cert");
        }

        if key.is_none() {
            println!("I: Writing debug key to {}...", key_path.to_string_lossy());
            std::fs::write(key_path, debug_key).expect("Failed to write debug key");
        }
        
        std::fs::write(apk_signer_path, apk_signer).expect("Failed to write APK signer");
        
        assert!(cert_path.exists(), "Certificate file does not exist");
        assert!(key_path.exists(), "Key file does not exist");
        assert!(apk_signer_path.exists(), "APK signer file does not exist");

        // Sign the APK using apksigner
        println!(
            "I: Signing APK file {}...",
            intermediate_apk_path.to_string_lossy()
        );
        let _ = run_java_jar(
            apk_signer_path.to_string_lossy().as_ref(),
            &[
                "sign",
                "--key",
                key_path.to_string_lossy().as_ref(),
                "--cert",
                cert_path.to_string_lossy().as_ref(),
                intermediate_apk_path.to_string_lossy().as_ref(),
            ],
        )
        .expect("Failed to unpack APK");
    }

    // Copy the signed APK to the output path replacing any existing file
    println!(
        "I: Copying signed APK to {}...",
        output_apk.to_string_lossy()
    );
    std::fs::copy(intermediate_apk_path, output_apk)
        .expect("Failed to move signed APK to output path");
}

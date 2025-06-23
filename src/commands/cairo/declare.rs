use crate::{cli::Cli, util::{self, OperationSummary, Timer, success, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli, network: &str) -> Result<()> {
    let _pkg_name = util::get_package_name(cli.pkg.as_ref())?;
    let mut summary = OperationSummary::new();

    if std::path::Path::new(".secrets").exists() {
        if let Err(e) = dotenv::from_filename(".secrets") {
            if cli.verbose {
                eprintln!("Warning: Failed to load .secrets file: {}", e);
            }
        }
    }

    let cairo_project_path = std::path::PathBuf::from("./contracts/cairo");
    let scarb_toml_path = cairo_project_path.join("Scarb.toml");
    if !cli.dry_run {
        util::validate_files_exist(&[scarb_toml_path]).map_err(enhance_error_with_suggestions)?;
    }

    let rpc_url = if network == "mainnet" {
        std::env::var("MAINNET_RPC_URL").map_err(|_| {
            create_smart_error(
                "MAINNET_RPC_URL environment variable not found",
                &["Add to your .secrets file: MAINNET_RPC_URL=https://starknet-mainnet.g.alchemy.com/starknet/version/rpc/v0_8/your_key", "Ensure the .secrets file is loaded in your environment"],
            )
        })?
    } else {
        std::env::var("SEPOLIA_RPC_URL").map_err(|_| {
            create_smart_error(
                "SEPOLIA_RPC_URL environment variable not found",
                &["Add to your .secrets file: SEPOLIA_RPC_URL=https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/v0_8/your_key", "Ensure the .secrets file is loaded in your environment"],
            )
        })?
    };

    let account_path = std::env::var("STARKNET_ACCOUNT").map_err(|_| {
        create_smart_error(
            "STARKNET_ACCOUNT environment variable not found",
            &["Add to your .secrets file: STARKNET_ACCOUNT=path/to/account.json", "This should point to your starkli account configuration file"],
        )
    })?;

    let keystore_path = std::env::var("STARKNET_KEYSTORE").map_err(|_| {
        create_smart_error(
            "STARKNET_KEYSTORE environment variable not found",
            &["Add to your .secrets file: STARKNET_KEYSTORE=path/to/keystore.json", "This should point to your starkli keystore file"],
        )
    })?;

    let compiled_contract_path = format!(
        "./contracts/cairo/target/dev/cairo_UltraStarknetZKHonkVerifier.contract_class.json"
    );

    let starkli_args = vec![
        "declare",
        &compiled_contract_path,
        "--rpc",
        &rpc_url,
        "--account",
        &account_path,
        "--keystore",
        &keystore_path,
        "--casm-file",
        "./contracts/cairo/target/dev/cairo_UltraStarknetZKHonkVerifier.compiled_contract_class.json",
    ];

    if cli.verbose {
        info!("Running: starkli {}", starkli_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: starkli {}", starkli_args.join(" "));
        return Ok(());
    }

    let declare_timer = Timer::start();
    let starkli_output = std::process::Command::new("starkli")
        .args(&starkli_args)
        .output()
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to run starkli declare: {}", e),
                &["Ensure starkli is installed and available in PATH", "Install with: curl -L https://get.starkli.sh | sh", "Check that the Cairo project was built correctly"],
            )
        })?;

    if !starkli_output.status.success() {
        let stderr = String::from_utf8_lossy(&starkli_output.stderr);
        let stdout = String::from_utf8_lossy(&starkli_output.stdout);
        return Err(create_smart_error(
            &format!("Starkli declare failed: {}{}", stdout, stderr),
            &["Check your account balance and network connectivity", "Verify the contract compilation was successful", "Ensure your private key and account address are correct"],
        ));
    }

    let stdout = String::from_utf8_lossy(&starkli_output.stdout);
    let mut class_hash_output = String::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("0x") && trimmed.len() >= 60 {
            class_hash_output = trimmed.to_string();
            break;
        }
    }

    if class_hash_output.is_empty() {
        return Err(create_smart_error(
            "Could not extract class hash from starkli output",
            &["The contract declaration may have failed", "Check the starkli output above for details", "Try running the command again"],
        ));
    }

    if !cli.quiet {
        println!("{}", success(&format!("Class hash: {}", class_hash_output)));
        let voyager_link = format!("https://voyager.online/class/{}", class_hash_output);
        println!("🔗 View on Voyager: {}", voyager_link);
    }

    let class_hash_file = std::path::Path::new("target/starknet/.bargo_class_hash");
    if let Some(parent) = class_hash_file.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            if cli.verbose {
                eprintln!("Warning: Failed to create target directory: {}", e);
            }
        }
    }
    if let Err(e) = std::fs::write(class_hash_file, &class_hash_output) {
        if cli.verbose {
            eprintln!("Warning: Failed to save class hash to file: {}", e);
        }
    }

    if !cli.quiet {
        println!(
            "{}",
            success(&format!("Contract declared successfully ({})", declare_timer.elapsed()))
        );
        let mut operation = "Cairo verifier contract declared on Starknet".to_string();
        if !class_hash_output.is_empty() {
            operation = format!("Cairo verifier contract declared on Starknet (class hash: {})", class_hash_output);
        }
        summary.add_operation(&operation);
        summary.print();
    }

    Ok(())
}

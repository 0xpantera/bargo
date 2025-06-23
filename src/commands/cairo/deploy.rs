use crate::{cli::Cli, util::{OperationSummary, Timer, success, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli, class_hash: Option<&str>) -> Result<()> {
    let mut summary = OperationSummary::new();

    let hash = match class_hash {
        Some(hash) => hash.to_string(),
        None => match std::fs::read_to_string("target/starknet/.bargo_class_hash") {
            Ok(saved_hash) => {
                let hash = saved_hash.trim().to_string();
                if !cli.quiet {
                    println!("{}", success(&format!("Using saved class hash: {}", hash)));
                }
                hash
            }
            Err(_) => {
                return Err(create_smart_error(
                    "Class hash is required for deployment",
                    &["Provide a class hash with --class-hash <HASH>", "Or run 'bargo cairo declare' first to save the class hash", "The declare command will save the class hash for deployment"],
                ));
            }
        },
    };

    if std::path::Path::new(".secrets").exists() {
        if let Err(e) = dotenv::from_filename(".secrets") {
            if cli.verbose {
                eprintln!("Warning: Failed to load .secrets file: {}", e);
            }
        }
    }

    let rpc_url = std::env::var("MAINNET_RPC_URL").map_err(|_| {
        create_smart_error(
            "MAINNET_RPC_URL environment variable not found",
            &["Add to your .secrets file: MAINNET_RPC_URL=https://starknet-mainnet.g.alchemy.com/starknet/version/rpc/v0_8/your_key", "Ensure the .secrets file is loaded in your environment"],
        )
    })?;

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

    let starkli_args = vec![
        "deploy",
        &hash,
        "--rpc",
        &rpc_url,
        "--account",
        &account_path,
        "--keystore",
        &keystore_path,
    ];

    if cli.verbose {
        info!("Running: starkli {}", starkli_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: starkli {}", starkli_args.join(" "));
        return Ok(());
    }

    let deploy_timer = Timer::start();
    let starkli_output = std::process::Command::new("starkli")
        .args(&starkli_args)
        .output()
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to run starkli deploy: {}", e),
                &["Ensure starkli is installed and available in PATH"],
            )
        })?;

    if !starkli_output.status.success() {
        let stderr = String::from_utf8_lossy(&starkli_output.stderr);
        let stdout = String::from_utf8_lossy(&starkli_output.stdout);
        return Err(create_smart_error(
            &format!("Starkli deploy failed: {}{}", stdout, stderr),
            &["Check your account balance and network connectivity"],
        ));
    }

    let output = String::from_utf8_lossy(&starkli_output.stdout);
    let mut contract_address = String::new();
    for line in output.lines() {
        if let Some(start) = line.find("0x") {
            let addr = &line[start..];
            if addr.len() >= 66 {
                contract_address = addr.split_whitespace().next().unwrap_or("").to_string();
                break;
            }
        }
    }

    if !contract_address.is_empty() {
        let address_file = std::path::Path::new("target/starknet/.bargo_contract_address");
        if let Some(parent) = address_file.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                if cli.verbose {
                    eprintln!("Warning: Failed to create target directory: {}", e);
                }
            }
        }
        if let Err(e) = std::fs::write(address_file, &contract_address) {
            if cli.verbose {
                eprintln!("Warning: Failed to save contract address to file: {}", e);
            }
        } else if !cli.quiet {
            println!("{}", success(&format!("Contract address: {}", contract_address)));
        }
    }

    if !cli.quiet {
        println!(
            "{}",
            success(&format!("Contract deployed successfully ({})", deploy_timer.elapsed()))
        );
        let mut operation = format!("Cairo verifier deployed with class hash: {}", hash);
        if !contract_address.is_empty() {
            operation = format!("Cairo verifier deployed at address: {}", contract_address);
        }
        summary.add_operation(&operation);
        summary.print();
    }

    Ok(())
}

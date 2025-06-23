use crate::{backends, cli::Cli, util::{OperationSummary, Timer, success, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    backends::foundry::ensure_available().map_err(enhance_error_with_suggestions)?;

    let rpc_url = std::env::var("RPC_URL").map_err(|_| {
        create_smart_error(
            "RPC_URL environment variable not found",
            &["Create a .env file in your project root", "Add: RPC_URL=https://your-rpc-endpoint.com", "For Sepolia: RPC_URL=https://eth-sepolia.g.alchemy.com/v2/your-key", "For Mainnet: RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-key"],
        )
    })?;

    let private_key = std::env::var("PRIVATE_KEY").map_err(|_| {
        create_smart_error(
            "PRIVATE_KEY environment variable not found",
            &["Add to your .env file: PRIVATE_KEY=0x...", "Make sure your private key starts with 0x", "Never commit your .env file to version control"],
        )
    })?;

    let contract_address_path = std::path::PathBuf::from("./target/bb/.bargo_contract_address");
    if !contract_address_path.exists() {
        return Err(create_smart_error(
            "Contract address not found",
            &["Run 'bargo evm deploy' first to deploy the verifier contract", "Ensure the deployment was successful"],
        ));
    }

    let contract_address = std::fs::read_to_string(&contract_address_path).map_err(|e| {
        create_smart_error(&format!("Failed to read contract address: {}", e), &["Check file permissions and try redeploying"])
    })?.trim().to_string();

    let calldata_path = std::path::PathBuf::from("./target/bb/calldata");
    if !calldata_path.exists() {
        return Err(create_smart_error(
            "Calldata not found",
            &["Run 'bargo evm calldata' first to generate calldata", "Ensure the proof and public inputs exist"],
        ));
    }

    let calldata = std::fs::read_to_string(&calldata_path).map_err(|e| {
        create_smart_error(&format!("Failed to read calldata: {}", e), &["Check file permissions and try regenerating calldata"])
    })?.trim().to_string();

    if !cli.quiet {
        println!("🚀 Verifying proof on-chain...");
        println!("📍 Contract: {}", contract_address);
    }

    let verify_args = vec![
        "send",
        &contract_address,
        "verify(bytes,bytes32[])",
        &calldata,
        "--rpc-url",
        &rpc_url,
        "--private-key",
        &private_key,
    ];

    if cli.verbose {
        info!("Running: cast {}", verify_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: cast {}", verify_args.join(" "));
        return Ok(());
    }

    let verify_timer = Timer::start();
    let (stdout, _stderr) = backends::foundry::run_cast_with_output(&verify_args).map_err(enhance_error_with_suggestions)?;

    let tx_hash = stdout
        .lines()
        .find(|line| line.contains("transactionHash"))
        .and_then(|line| line.split('"').nth(3))
        .or_else(|| {
            stdout
                .lines()
                .find(|line| line.starts_with("0x") && line.len() == 66)
                .map(|line| line.trim())
        })
        .ok_or_else(|| {
            create_smart_error(
                "Failed to parse transaction hash from cast output",
                &["Check that the transaction was successful", "Verify your RPC_URL and PRIVATE_KEY are correct", "Ensure you have sufficient funds for gas", "Check that the contract address is correct"],
            )
        })?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!("Proof verified on-chain successfully ({})", verify_timer.elapsed()))
        );
        println!("📋 Transaction hash: {}", tx_hash);

        if rpc_url.contains("sepolia") {
            println!("🔗 View on Etherscan: https://sepolia.etherscan.io/tx/{}", tx_hash);
        } else if rpc_url.contains("mainnet") || !rpc_url.contains("sepolia") {
            println!("🔗 View on Etherscan: https://etherscan.io/tx/{}", tx_hash);
        }

        summary.add_operation(&format!("Proof verified on-chain (tx: {})", tx_hash));
        summary.print();
    }

    Ok(())
}

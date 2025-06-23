use crate::{backends, cli::Cli, util::{OperationSummary, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli, network: &str) -> Result<()> {
    let summary = OperationSummary::new();

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

    let verifier_path = std::path::PathBuf::from("./contracts/evm/src/Verifier.sol");
    if !verifier_path.exists() {
        return Err(create_smart_error(
            "Verifier.sol not found",
            &["Run 'bargo evm gen' first to generate the Solidity verifier", "Ensure the Foundry project was created successfully"],
        ));
    }

    if !cli.quiet {
        println!("🚀 Deploying verifier contract to {}...", network);
    }

    let deploy_args = vec![
        "create",
        "--rpc-url",
        &rpc_url,
        "--private-key",
        &private_key,
        "--legacy",
        "--constructor-args",
        "",
        "--broadcast",
        "--verify",
        "--unlocked",
        "--build-info",
        "contracts/evm/out/Verifier.sol/Verifier.json",
    ];

    if cli.verbose {
        info!("Running: forge {}", deploy_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: forge {}", deploy_args.join(" "));
        return Ok(());
    }

    backends::foundry::run_forge(&deploy_args).map_err(enhance_error_with_suggestions)?;

    let contract_address_output = std::path::Path::new("./broadcast/Verifier.sol/0/run.json");
    let contract_address = std::fs::read_to_string(contract_address_output).map_err(|e| {
        create_smart_error(&format!("Failed to read deployment output: {}", e), &["Check that the deployment succeeded"])
    })?;
    let address_json: serde_json::Value = serde_json::from_str(&contract_address)?;
    let contract_address = address_json["returns"]["contract"]["address"].as_str().unwrap_or("").to_string();

    std::fs::write("./target/bb/.bargo_contract_address", &contract_address).map_err(|e| {
        create_smart_error(&format!("Failed to save contract address: {}", e), &["Check directory permissions"])
    })?;

    if !cli.quiet {
        summary.print();
        println!();
        println!("🎯 Next steps:");
        println!("  1. Generate calldata: bargo evm calldata");
        println!("  2. Verify proof on-chain: bargo evm verify-onchain");
    }

    Ok(())
}

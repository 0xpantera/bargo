use crate::cli::Cli;
use color_eyre::Result;

pub fn run(cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("🔍 Checking system dependencies...\n");
    }

    let mut all_good = true;

    match which::which("nargo") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ nargo: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("❌ nargo: not found");
                println!("   Install from: https://noir-lang.org/docs/getting_started/installation/");
            }
            all_good = false;
        }
    }

    match which::which("bb") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ bb: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("❌ bb: not found");
                println!("   Install from: https://github.com/AztecProtocol/aztec-packages");
            }
            all_good = false;
        }
    }

    match which::which("garaga") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ garaga: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("⚠️  garaga: not found (optional - needed for Cairo features)");
                println!("   Install with: pipx install garaga");
                println!("   Requires Python 3.10+");
            }
        }
    }

    match which::which("forge") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ forge: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("⚠️  forge: not found (optional - needed for EVM features)");
                println!("   Install with: curl -L https://foundry.paradigm.xyz | bash");
                println!("   Then run: foundryup");
            }
        }
    }

    match which::which("cast") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ cast: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("⚠️  cast: not found (optional - needed for EVM features)");
                println!("   Install with: curl -L https://foundry.paradigm.xyz | bash");
                println!("   Then run: foundryup");
            }
        }
    }

    if !cli.quiet {
        println!();
        if all_good {
            println!("🎉 All required dependencies are available!");
            println!("   You can use all bargo features.");
        } else {
            println!("🚨 Some required dependencies are missing.");
            println!("   Core features require: nargo + bb");
            println!("   EVM deployment features also require: forge + cast");
            println!("   Cairo features also require: garaga");
        }
    }

    if !all_good {
        std::process::exit(1);
    }

    Ok(())
}

use color_eyre::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Backend flavour for artifact generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavour {
    /// Barretenberg backend (EVM/Solidity)
    Bb,
    /// Starknet backend (Cairo)
    Starknet,
}

/// Find the project root by walking up the directory tree looking for Nargo.toml
pub fn find_project_root(current_path: &Path) -> Result<PathBuf> {
    for path in current_path.ancestors() {
        let nargo_toml = path.join("Nargo.toml");
        if nargo_toml.exists() {
            debug!("Found Nargo.toml at: {}", nargo_toml.display());
            return Ok(path.to_path_buf());
        }
    }

    Err(color_eyre::eyre::eyre!(
        "Could not find Nargo.toml in current directory or any parent directory.\n\
         Make sure you're running bargo from within a Noir project."
    ))
}

/// Get the package name from Nargo.toml, with optional override
pub fn get_package_name(pkg_override: Option<&String>) -> Result<String> {
    if let Some(pkg_name) = pkg_override {
        debug!("Using package name override: {}", pkg_name);
        return Ok(pkg_name.clone());
    }

    let current_dir = std::env::current_dir()?;
    let project_root = find_project_root(&current_dir)?;
    let nargo_toml_path = project_root.join("Nargo.toml");

    parse_package_name(&nargo_toml_path)
}

/// Parse the package name from a Nargo.toml file
pub fn parse_package_name(nargo_toml_path: &Path) -> Result<String> {
    let toml_content = std::fs::read_to_string(nargo_toml_path).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to read Nargo.toml at {}: {}",
            nargo_toml_path.display(),
            e
        )
    })?;

    let config: TomlConfig = toml::from_str(&toml_content).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to parse Nargo.toml at {}: {}",
            nargo_toml_path.display(),
            e
        )
    })?;

    match config {
        TomlConfig::Package { package } => package.name.ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "Missing 'name' field in [package] section of Nargo.toml at {}",
                nargo_toml_path.display()
            )
        }),
        TomlConfig::Workspace { .. } => {
            warn!("Found workspace Nargo.toml, using directory name as package name");
            let dir_name = nargo_toml_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "Could not determine package name from workspace directory"
                    )
                })?;
            Ok(dir_name)
        }
    }
}

/// Get the target directory path for a specific backend flavour
pub fn target_dir(flavour: Flavour) -> PathBuf {
    match flavour {
        Flavour::Bb => PathBuf::from("target/bb"),
        Flavour::Starknet => PathBuf::from("target/starknet"),
    }
}

/// Get the target directory path (defaults to bb backend for compatibility)
pub fn workspace_target() -> PathBuf {
    target_dir(Flavour::Bb)
}

/// Get the bytecode file path for a package with specific backend flavour
pub fn get_bytecode_path(pkg_name: &str, flavour: Flavour) -> PathBuf {
    target_dir(flavour).join(format!("{}.json", pkg_name))
}

/// Get the witness file path for a package with specific backend flavour
pub fn get_witness_path(pkg_name: &str, flavour: Flavour) -> PathBuf {
    target_dir(flavour).join(format!("{}.gz", pkg_name))
}

/// Get the proof file path for specific backend flavour
pub fn get_proof_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("proof")
}

/// Get the verification key file path for specific backend flavour
pub fn get_vk_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("vk")
}

/// Get the public inputs file path for specific backend flavour
pub fn get_public_inputs_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("public_inputs")
}

/// Organize build artifacts by moving nargo output to appropriate flavour directory
pub fn organize_build_artifacts(pkg_name: &str, flavour: Flavour) -> Result<()> {
    // Create the target directory for the flavour if it doesn't exist
    let flavour_dir = target_dir(flavour);
    std::fs::create_dir_all(&flavour_dir).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to create target directory {}: {}",
            flavour_dir.display(),
            e
        )
    })?;

    // Move bytecode file from target/ to target/flavour/
    let source_bytecode = PathBuf::from("target").join(format!("{}.json", pkg_name));
    let dest_bytecode = get_bytecode_path(pkg_name, flavour);

    if source_bytecode.exists() {
        std::fs::rename(&source_bytecode, &dest_bytecode).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_bytecode.display(),
                dest_bytecode.display(),
                e
            )
        })?;
        debug!(
            "Moved bytecode: {} -> {}",
            source_bytecode.display(),
            dest_bytecode.display()
        );
    }

    // Move witness file from target/ to target/flavour/
    let source_witness = PathBuf::from("target").join(format!("{}.gz", pkg_name));
    let dest_witness = get_witness_path(pkg_name, flavour);

    if source_witness.exists() {
        std::fs::rename(&source_witness, &dest_witness).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_witness.display(),
                dest_witness.display(),
                e
            )
        })?;
        debug!(
            "Moved witness: {} -> {}",
            source_witness.display(),
            dest_witness.display()
        );
    }

    Ok(())
}

/// Organize bb artifacts by moving bb output to appropriate flavour directory
pub fn organize_bb_artifacts(flavour: Flavour) -> Result<()> {
    // Create the target directory for the flavour if it doesn't exist
    let flavour_dir = target_dir(flavour);
    std::fs::create_dir_all(&flavour_dir).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to create target directory {}: {}",
            flavour_dir.display(),
            e
        )
    })?;

    // Move proof file from target/ to target/flavour/
    let source_proof = PathBuf::from("target/proof");
    let dest_proof = get_proof_path(flavour);

    if source_proof.exists() {
        std::fs::rename(&source_proof, &dest_proof).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_proof.display(),
                dest_proof.display(),
                e
            )
        })?;
        debug!(
            "Moved proof: {} -> {}",
            source_proof.display(),
            dest_proof.display()
        );
    }

    // Move vk file from target/ to target/flavour/
    let source_vk = PathBuf::from("target/vk");
    let dest_vk = get_vk_path(flavour);

    if source_vk.exists() {
        std::fs::rename(&source_vk, &dest_vk).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_vk.display(),
                dest_vk.display(),
                e
            )
        })?;
        debug!("Moved vk: {} -> {}", source_vk.display(), dest_vk.display());
    }

    // Move public_inputs file from target/ to target/flavour/
    let source_public_inputs = PathBuf::from("target/public_inputs");
    let dest_public_inputs = get_public_inputs_path(flavour);

    if source_public_inputs.exists() {
        std::fs::rename(&source_public_inputs, &dest_public_inputs).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_public_inputs.display(),
                dest_public_inputs.display(),
                e
            )
        })?;
        debug!(
            "Moved public_inputs: {} -> {}",
            source_public_inputs.display(),
            dest_public_inputs.display()
        );
    }

    Ok(())
}

/// Validate that required files exist for a given operation
pub fn validate_files_exist(files: &[PathBuf]) -> Result<()> {
    let mut missing_files = Vec::new();

    for file in files {
        if !file.exists() {
            missing_files.push(file);
        }
    }

    if !missing_files.is_empty() {
        let missing_list = missing_files
            .iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n");

        return Err(color_eyre::eyre::eyre!(
            "Required files are missing:\n{}\n\nTry running `bargo build` first.",
            missing_list
        ));
    }

    Ok(())
}

/// Check if source files are newer than target files (for smart rebuilds)
pub fn needs_rebuild(pkg_name: &str) -> Result<bool> {
    let current_dir = std::env::current_dir()?;
    let project_root = find_project_root(&current_dir)?;

    // Check if target files exist
    let bytecode_path = get_bytecode_path(pkg_name, Flavour::Bb);
    let witness_path = get_witness_path(pkg_name, Flavour::Bb);

    if !bytecode_path.exists() || !witness_path.exists() {
        debug!("Target files don't exist, rebuild needed");
        return Ok(true);
    }

    // Get the oldest target file time
    let bytecode_time = std::fs::metadata(&bytecode_path)?.modified()?;
    let witness_time = std::fs::metadata(&witness_path)?.modified()?;
    let target_time = bytecode_time.min(witness_time);

    // Check Nargo.toml modification time
    let nargo_toml = project_root.join("Nargo.toml");
    if nargo_toml.exists() {
        let nargo_time = std::fs::metadata(&nargo_toml)?.modified()?;
        if nargo_time > target_time {
            debug!("Nargo.toml is newer than target files, rebuild needed");
            return Ok(true);
        }
    }

    // Check if any source files are newer
    let src_dir = project_root.join("src");
    if src_dir.exists() {
        if is_dir_newer_than(&src_dir, target_time)? {
            debug!("Source files are newer than target files, rebuild needed");
            return Ok(true);
        }
    }

    debug!("Target files are up to date");
    Ok(false)
}

/// Recursively check if any file in a directory is newer than the given time
fn is_dir_newer_than(dir: &Path, target_time: std::time::SystemTime) -> Result<bool> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_time = std::fs::metadata(&path)?.modified()?;
            if file_time > target_time {
                return Ok(true);
            }
        } else if path.is_dir() {
            if is_dir_newer_than(&path, target_time)? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Timer for tracking operation duration
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Get elapsed time as a formatted string
    pub fn elapsed(&self) -> String {
        let duration = self.start.elapsed();
        if duration.as_secs() > 0 {
            format!("{:.1}s", duration.as_secs_f64())
        } else {
            format!("{}ms", duration.as_millis())
        }
    }
}

/// Format file size in human-readable format
pub fn format_file_size(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let size = metadata.len();
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            }
        }
        Err(_) => "unknown size".to_string(),
    }
}

/// Enhanced error with suggestions for common issues
pub fn enhance_error_with_suggestions(error: color_eyre::eyre::Error) -> color_eyre::eyre::Error {
    let error_msg = format!("{}", error);

    // Check for common error patterns and add suggestions
    if error_msg.contains("Required files are missing") {
        if error_msg.contains(".json") || error_msg.contains(".gz") {
            return color_eyre::eyre::eyre!(
                "{}\n\n💡 Suggestions:\n   • Run `bargo build` to generate bytecode and witness files\n   • Check if you're in the correct Noir project directory\n   • Verify that Nargo.toml exists in the current directory",
                error
            );
        }
        if error_msg.contains("proof") || error_msg.contains("vk") {
            return color_eyre::eyre::eyre!(
                "{}\n\n💡 Suggestions:\n   • Run `bargo prove` to generate proof and verification key\n   • Check if the proving step completed successfully\n   • Try running `bargo clean` and rebuilding from scratch",
                error
            );
        }
    }

    if error_msg.contains("Could not find Nargo.toml") {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Make sure you're in a Noir project directory\n   • Initialize a new project with `nargo new <project_name>`\n   • Check if you're in a subdirectory - try running from the project root",
            error
        );
    }

    if error_msg.contains("Failed to parse Nargo.toml") {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Check Nargo.toml syntax - it should be valid TOML format\n   • Ensure the [package] section has a 'name' field\n   • Compare with a working Nargo.toml from another project",
            error
        );
    }

    if error_msg.contains("nargo")
        && (error_msg.contains("not found") || error_msg.contains("command not found"))
    {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Install nargo: `curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash`\n   • Add nargo to your PATH\n   • Verify installation with `nargo --version`",
            error
        );
    }

    if error_msg.contains("bb")
        && (error_msg.contains("not found") || error_msg.contains("command not found"))
    {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Install bb (Barretenberg): check Aztec installation docs\n   • Add bb to your PATH\n   • Verify installation with `bb --version`",
            error
        );
    }

    // Return original error if no patterns match
    error
}

/// Format operation result with file size and timing
pub fn format_operation_result(operation: &str, file_path: &Path, timer: &Timer) -> String {
    let size = format_file_size(file_path);
    let elapsed = timer.elapsed();
    format!(
        "{} → {} ({}, {})",
        operation,
        file_path.display(),
        size,
        elapsed
    )
}

/// Create a smart error with context and suggestions
pub fn create_smart_error(message: &str, suggestions: &[&str]) -> color_eyre::eyre::Error {
    let mut error_msg = format!("❌ {}", message);

    if !suggestions.is_empty() {
        error_msg.push_str("\n\n💡 Suggestions:");
        for suggestion in suggestions {
            error_msg.push_str(&format!("\n   • {}", suggestion));
        }
    }

    color_eyre::eyre::eyre!(error_msg)
}

/// ANSI color codes for terminal output
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const GREEN: &str = "\x1b[32m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const GRAY: &str = "\x1b[90m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
}

/// Format text with color
pub fn colorize(text: &str, color: &str) -> String {
    if std::env::var("NO_COLOR").is_ok() || !atty::is(atty::Stream::Stdout) {
        text.to_string()
    } else {
        format!("{}{}{}", color, text, colors::RESET)
    }
}

/// Create success message with green color
pub fn success(text: &str) -> String {
    colorize(&format!("✅ {}", text), colors::BRIGHT_GREEN)
}

/// Create error message with red color
pub fn error(text: &str) -> String {
    colorize(&format!("❌ {}", text), colors::RED)
}

/// Create warning message with yellow color
pub fn warning(text: &str) -> String {
    colorize(&format!("⚠️ {}", text), colors::YELLOW)
}

/// Create info message with blue color
pub fn info(text: &str) -> String {
    colorize(&format!("ℹ️ {}", text), colors::BRIGHT_BLUE)
}

/// Create path text with cyan color
pub fn path(text: &str) -> String {
    colorize(text, colors::BRIGHT_CYAN)
}

/// ASCII art banners for different operations
pub fn print_banner(operation: &str) {
    let banner = match operation {
        "build" => {
            "┌─────────────────────────────────┐\n\
             │ 🔨 BUILDING NOIR CIRCUIT       │\n\
             └─────────────────────────────────┘"
        }
        "prove" => {
            "┌─────────────────────────────────┐\n\
             │ 🔐 GENERATING PROOF & VK        │\n\
             └─────────────────────────────────┘"
        }
        "verify" => {
            "┌─────────────────────────────────┐\n\
             │ ✅ VERIFYING PROOF              │\n\
             └─────────────────────────────────┘"
        }
        "solidity" => {
            "┌─────────────────────────────────┐\n\
             │ 📄 GENERATING SOLIDITY VERIFIER │\n\
             └─────────────────────────────────┘"
        }
        "clean" => {
            "┌─────────────────────────────────┐\n\
             │ 🧹 CLEANING BUILD ARTIFACTS    │\n\
             └─────────────────────────────────┘"
        }
        "check" => {
            "┌─────────────────────────────────┐\n\
             │ 🔍 CHECKING CIRCUIT SYNTAX      │\n\
             └─────────────────────────────────┘"
        }
        _ => {
            "┌─────────────────────────────────┐\n\
             │ 🚀 RUNNING BARGO OPERATION      │\n\
             └─────────────────────────────────┘"
        }
    };

    println!("{}", colorize(banner, colors::BRIGHT_BLUE));
}

/// Print operation summary with colored output
pub struct OperationSummary {
    operations: Vec<String>,
    start_time: std::time::Instant,
}

impl OperationSummary {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn add_operation(&mut self, operation: &str) {
        self.operations.push(operation.to_string());
    }

    pub fn print(&self) {
        if self.operations.is_empty() {
            return;
        }

        let total_time = self.start_time.elapsed();
        let time_str = if total_time.as_secs() > 0 {
            format!("{:.1}s", total_time.as_secs_f64())
        } else {
            format!("{}ms", total_time.as_millis())
        };

        println!("\n{}", colorize("🎉 Summary:", colors::BOLD));
        for operation in &self.operations {
            println!(
                "   {}",
                colorize(&format!("• {}", operation), colors::GREEN)
            );
        }
        println!(
            "   {}",
            colorize(&format!("Total time: {}", time_str), colors::GRAY)
        );
    }
}

/// Simplified Nargo.toml configuration structure
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlConfig {
    Package { package: PackageMetadata },
    Workspace { workspace: WorkspaceMetadata },
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
    name: Option<String>,
    #[serde(alias = "type")]
    package_type: Option<String>,
    version: Option<String>,
    authors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceMetadata {
    members: Vec<String>,
    #[serde(alias = "default-member")]
    default_member: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(temp_dir: &TempDir, name: &str) -> PathBuf {
        let project_dir = temp_dir.path().join("test_project");
        fs::create_dir_all(&project_dir).unwrap();

        let nargo_toml = format!(
            r#"[package]
name = "{}"
type = "bin"
authors = ["test"]
"#,
            name
        );

        fs::write(project_dir.join("Nargo.toml"), nargo_toml).unwrap();

        // Create src directory and main.nr
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("main.nr"), "fn main() {}").unwrap();

        project_dir
    }

    #[test]
    fn test_find_project_root() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project(&temp_dir, "test_pkg");
        let src_dir = project_dir.join("src");

        let found_root = find_project_root(&src_dir).unwrap();
        assert_eq!(found_root, project_dir);
    }

    #[test]
    fn test_parse_package_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project(&temp_dir, "my_package");
        let nargo_toml = project_dir.join("Nargo.toml");

        let name = parse_package_name(&nargo_toml).unwrap();
        assert_eq!(name, "my_package");
    }

    #[test]
    fn test_get_package_name_with_override() {
        let override_name = "override_pkg".to_string();
        let name = get_package_name(Some(&override_name)).unwrap();
        assert_eq!(name, "override_pkg");
    }

    #[test]
    fn test_path_helpers() {
        assert_eq!(get_bytecode_path("test", Flavour::Bb), PathBuf::from("target/bb/test.json"));
        assert_eq!(get_witness_path("test", Flavour::Bb), PathBuf::from("target/bb/test.gz"));
        assert_eq!(get_proof_path(Flavour::Bb), PathBuf::from("target/bb/proof"));
        assert_eq!(get_vk_path(Flavour::Bb), PathBuf::from("target/bb/vk"));
    }

    #[test]
    fn test_needs_rebuild_no_target() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project(&temp_dir, "test_pkg");

        // Change to project directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let needs_rebuild = needs_rebuild("test_pkg").unwrap();
        assert!(needs_rebuild);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}

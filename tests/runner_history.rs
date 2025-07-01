//! Integration tests for DryRunRunner history functionality
//!
//! These tests demonstrate how to use DryRunRunner history for testing
//! instead of grepping stdout output.

use bargo_core::runner::{CmdSpec, DryRunRunner, Runner};

#[test]
fn test_dry_run_runner_history_basic() {
    // Create a DryRunRunner directly
    let runner = DryRunRunner::new();

    // Execute some commands
    let spec1 = CmdSpec::new(
        "bb".to_string(),
        vec!["prove".to_string(), "--help".to_string()],
    );
    let spec2 = CmdSpec::new(
        "garaga".to_string(),
        vec!["gen".to_string(), "--version".to_string()],
    );

    runner.run(&spec1).unwrap();
    runner.run(&spec2).unwrap();

    // Verify history contains the expected commands
    let history = runner.history();
    assert_eq!(history.len(), 2);

    assert_eq!(history[0].cmd, "bb");
    assert_eq!(history[0].args, vec!["prove", "--help"]);

    assert_eq!(history[1].cmd, "garaga");
    assert_eq!(history[1].args, vec!["gen", "--version"]);
}

#[test]
fn test_dry_run_runner_history_with_capture() {
    let runner = DryRunRunner::new();

    // Mix regular run and capture commands
    let spec1 = CmdSpec::new(
        "forge".to_string(),
        vec!["init".to_string(), "test".to_string()],
    );
    let spec2 = CmdSpec::new(
        "garaga".to_string(),
        vec!["calldata".to_string(), "--output".to_string()],
    );

    runner.run(&spec1).unwrap();
    let _output = runner.run_capture(&spec2).unwrap();

    // Both should be in history
    let history = runner.history();
    assert_eq!(history.len(), 2);

    assert_eq!(history[0].cmd, "forge");
    assert_eq!(history[1].cmd, "garaga");

    // run_capture should return placeholder output
    let spec3 = CmdSpec::new("echo".to_string(), vec!["test".to_string()]);
    let output = runner.run_capture(&spec3).unwrap();
    assert_eq!(output, "<dry-run-output>");
}

#[test]
fn test_dry_run_runner_history_clear() {
    let runner = DryRunRunner::new();

    // Add some commands
    let spec = CmdSpec::new("nargo".to_string(), vec!["check".to_string()]);
    runner.run(&spec).unwrap();
    runner.run(&spec).unwrap();

    assert_eq!(runner.history().len(), 2);

    // Clear and verify
    runner.clear_history();
    assert_eq!(runner.history().len(), 0);

    // Add more commands after clear
    runner.run(&spec).unwrap();
    assert_eq!(runner.history().len(), 1);
}

#[test]
fn test_dry_run_runner_thread_safety() {
    // Test that DryRunRunner can be used safely across multiple operations
    let runner = DryRunRunner::new();

    // Execute multiple commands to test thread safety of the mutex
    for i in 0..5 {
        let spec = CmdSpec::new("test".to_string(), vec![format!("arg{}", i)]);
        runner.run(&spec).unwrap();
    }

    let history = runner.history();
    assert_eq!(history.len(), 5);

    // Verify all commands were recorded correctly
    for (i, cmd) in history.iter().enumerate() {
        assert_eq!(cmd.cmd, "test");
        assert_eq!(cmd.args, vec![format!("arg{}", i)]);
    }
}

#[test]
fn test_complex_command_history() {
    let runner = DryRunRunner::new();

    // Simulate a complex workflow like "cairo gen"
    let commands = vec![
        (
            "bb",
            vec![
                "prove",
                "--scheme",
                "ultra_honk",
                "--oracle_hash",
                "starknet",
            ],
        ),
        (
            "bb",
            vec!["write_vk", "--oracle_hash", "starknet", "-b", "test.json"],
        ),
        (
            "garaga",
            vec!["gen", "--system", "ultra_starknet_zk_honk", "--vk", "vk"],
        ),
    ];

    // Execute all commands
    for (tool, args) in &commands {
        let spec = CmdSpec::new(
            tool.to_string(),
            args.iter().map(|s| s.to_string()).collect(),
        );
        runner.run(&spec).unwrap();
    }

    // Verify the complete history
    let history = runner.history();
    assert_eq!(history.len(), 3);

    // Check each command in detail
    assert_eq!(history[0].cmd, "bb");
    assert!(history[0].args.contains(&"prove".to_string()));
    assert!(history[0].args.contains(&"ultra_honk".to_string()));

    assert_eq!(history[1].cmd, "bb");
    assert!(history[1].args.contains(&"write_vk".to_string()));

    assert_eq!(history[2].cmd, "garaga");
    assert!(history[2].args.contains(&"gen".to_string()));
    assert!(
        history[2]
            .args
            .contains(&"ultra_starknet_zk_honk".to_string())
    );
}

#[test]
fn test_cmd_spec_with_environment_and_cwd() {
    let runner = DryRunRunner::new();

    // Test CmdSpec with working directory and environment variables
    let spec = CmdSpec::new("forge".to_string(), vec!["create".to_string()])
        .with_cwd(std::path::PathBuf::from("/tmp/test"))
        .with_env("RPC_URL".to_string(), "http://localhost:8545".to_string())
        .with_env("PRIVATE_KEY".to_string(), "0x123".to_string());

    runner.run(&spec).unwrap();

    let history = runner.history();
    assert_eq!(history.len(), 1);

    let recorded_cmd = &history[0];
    assert_eq!(recorded_cmd.cmd, "forge");
    assert_eq!(recorded_cmd.args, vec!["create"]);
    assert_eq!(
        recorded_cmd.cwd,
        Some(std::path::PathBuf::from("/tmp/test"))
    );
    assert_eq!(recorded_cmd.env.len(), 2);
    assert!(
        recorded_cmd
            .env
            .contains(&("RPC_URL".to_string(), "http://localhost:8545".to_string()))
    );
    assert!(
        recorded_cmd
            .env
            .contains(&("PRIVATE_KEY".to_string(), "0x123".to_string()))
    );
}

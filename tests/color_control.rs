use std::process::Command;

fn execute_test(env_key: &str, env_val: &str) {
    let mut child_proc = Command::new("cargo")
        .args(["run", "--example", "compile_time_config"])
        .env(env_key, env_val)
        .spawn()
        .expect("Cargo command failed to start");

    let ecode = child_proc.wait().expect("failed to wait on child");

    assert!(ecode.success());
}

// Maintaining as a single test to avoid blocking calls to the package cache
#[test]
fn test_no_color() {
    let keys = vec!["NO_COLOR", "CLICOLOR_FORCE", "CLICOLOR"];

    for key in keys {
        execute_test(key, "1");
        execute_test(key, "0");
    }
}

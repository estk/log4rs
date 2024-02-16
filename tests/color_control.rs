use std::process::Command;

fn execute_test(env_key: &str, env_val: &str) {
    let mut child_proc = Command::new("cargo")
        .args(&["run", "--example", "color_control"])
        .env(env_key, env_val)
        .spawn()
        .expect("Cargo command failed to start");

    let ecode = child_proc.wait().expect("failed to wait on child");

    assert!(ecode.success());
}

// Maintaining as a single test to avoid blocking calls to the package cache
#[test]
fn test_single_var() {
    let keys = vec!["NO_COLOR", "CLICOLOR_FORCE", "CLICOLOR"];

    for key in keys {
        execute_test(key, "1");
        execute_test(key, "0");
    }
}

#[test]
fn test_no_color_vs_force() {
    let mut child_proc = Command::new("cargo")
        .args(&["run", "--example", "color_control"])
        .env("NO_COLOR", "1")
        .env("CLICOLOR_FORCE", "1")
        .spawn()
        .expect("Cargo command failed to start");

    let ecode = child_proc.wait().expect("failed to wait on child");

    assert!(ecode.success());
}

#[test]
fn test_no_color_vs_regular() {
    let mut child_proc = Command::new("cargo")
        .args(&["run", "--example", "color_control"])
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "1")
        .spawn()
        .expect("Cargo command failed to start");

    let ecode = child_proc.wait().expect("failed to wait on child");

    assert!(ecode.success());
}

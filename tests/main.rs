#[test]
#[ignore = "This is an example integration test"]
fn test_binary_output() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "install"])
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("{}", stdout);
    assert!(stdout.contains("leuko"));
}

#[test]
fn shim() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "audit", "zod"])
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("{}", stdout);
    assert!(stdout.contains("Auditing zod"));
}

use std::ffi::OsString;
use std::process::{Command, ExitCode};

const ALLOWED_EDGES: &[(&str, &[&str])] = &[("xtask", &[])];

fn main() -> ExitCode {
    let passed = match std::env::args().nth(1).as_deref() {
        Some("ci") => ci(),
        Some("check-deps") => check_deps(),
        _ => {
            eprintln!("usage: cargo xtask <ci|check-deps>");
            false
        }
    };
    if passed { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn ci() -> bool {
    run(&["fmt", "--all", "--check"])
        && run(&["clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"])
        && check_deps()
        && run(&["test", "--workspace", "--locked"])
}

fn cargo(args: &[&str]) -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")));
    command.args(args);
    command
}

fn run(args: &[&str]) -> bool {
    eprintln!("==> cargo {}", args.join(" "));
    match cargo(args).status() {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("{error}");
            false
        }
    }
}

fn check_deps() -> bool {
    eprintln!("==> check-deps");
    let Some(metadata) = metadata() else { return false };
    let Some(packages) = metadata["packages"].as_array() else {
        eprintln!("cargo metadata has no packages");
        return false;
    };
    let members: Vec<&str> = packages.iter().filter_map(|package| package["name"].as_str()).collect();
    let mut violations = Vec::new();
    for package in packages {
        let Some(name) = package["name"].as_str() else {
            violations.push("a package has no name".to_owned());
            continue;
        };
        let Some((_, allowed)) = ALLOWED_EDGES.iter().find(|(crate_name, _)| *crate_name == name) else {
            violations.push(format!("{name} is not in the dependency table"));
            continue;
        };
        let Some(dependencies) = package["dependencies"].as_array() else {
            violations.push(format!("{name} has no dependency list"));
            continue;
        };
        for dependency in dependencies.iter().filter_map(|dependency| dependency["name"].as_str()) {
            if members.contains(&dependency) && !allowed.contains(&dependency) {
                violations.push(format!("{name} -> {dependency}"));
            }
        }
    }
    for violation in &violations {
        eprintln!("forbidden workspace edge: {violation}");
    }
    violations.is_empty()
}

fn metadata() -> Option<serde_json::Value> {
    let output = match cargo(&["metadata", "--format-version", "1", "--no-deps"]).output() {
        Ok(output) if output.status.success() => output.stdout,
        Ok(output) => {
            eprintln!("cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr));
            return None;
        }
        Err(error) => {
            eprintln!("cargo metadata: {error}");
            return None;
        }
    };
    match serde_json::from_slice(&output) {
        Ok(metadata) => Some(metadata),
        Err(error) => {
            eprintln!("cargo metadata is not JSON: {error}");
            None
        }
    }
}

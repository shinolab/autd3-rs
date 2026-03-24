use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "xtask", about = "Workspace tasks for autd3-rs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Format code
    Format,
    /// Clean workspace
    Clean,
    /// Format, lint, build, and test
    Check {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build workspace
    Build {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Lint code
    Lint {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build docs
    Doc {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run all tests
    Test {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run lib tests
    TestLib {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run doc tests
    TestDoc {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a binary
    Run {
        bin: String,
        #[arg(long, short, default_value_t = false)]
        debug: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Coverage
    Cov {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Update version
    UpdateVersion { version: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let workspace_root = project_root()?;
    env::set_current_dir(&workspace_root)?;

    match cli.command {
        Commands::Format => {
            cmd("cargo", &["fmt"])?;
        }
        Commands::Clean => {
            cmd("cargo", &["clean"])?;
        }
        Commands::Check { args } => {
            cmd("cargo", &["fmt"])?;
            run_lint(&args)?;
            run_build(&args)?;
            run_test_lib(&args)?;
            run_test_doc(&args)?;
        }
        Commands::Build { args } => {
            run_build(&args)?;
        }
        Commands::Lint { args } => {
            run_lint(&args)?;
        }
        Commands::Doc { args } => {
            let mut c = Command::new("cargo");
            c.arg("+nightly")
                .arg("doc")
                .arg("--workspace")
                .arg("--no-deps");
            c.args(args);
            c.env("RUSTDOCFLAGS", "--cfg docsrs -D warnings");
            run_cmd(c)?;
        }
        Commands::Test { args } => {
            run_test_lib(&args)?;
            run_test_doc(&args)?;
        }
        Commands::TestLib { args } => {
            run_test_lib(&args)?;
        }
        Commands::TestDoc { args } => {
            run_test_doc(&args)?;
        }
        Commands::Run { bin, debug, args } => {
            let mut cmd_args = vec![
                "run".to_string(),
                "--bin".to_string(),
                bin.clone(),
                "--no-default-features".to_string(),
                "--features".to_string(),
                bin,
            ];
            if !debug {
                cmd_args.push("--release".to_string());
            }
            cmd_args.extend(args);
            cmd(
                "cargo",
                &cmd_args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )?;
        }
        Commands::Cov { args } => {
            run_cov(&args)?;
        }
        Commands::UpdateVersion { version } => {
            run_update_version(&version)?;
        }
    }
    Ok(())
}

fn project_root() -> Result<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = Path::new(manifest_dir).parent().context("no parent")?;
    Ok(root.to_path_buf())
}

fn cmd(program: &str, args: &[&str]) -> Result<()> {
    let mut c = Command::new(program);
    c.args(args);
    run_cmd(c)
}

fn run_cmd(mut cmd: Command) -> Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Command `{:?}` failed with status {}", cmd, status);
    }
    Ok(())
}

fn run_build(args: &[String]) -> Result<()> {
    let mut c = vec!["build", "--workspace", "--bins", "--exclude", "xtask"];
    c.extend(args.iter().map(|s| s.as_str()));
    cmd("cargo", &c)
}

fn run_lint(args: &[String]) -> Result<()> {
    let mut c = vec!["clippy", "--tests", "--workspace", "--exclude", "xtask"];
    c.extend(args.iter().map(|s| s.as_str()));
    c.extend(&["--", "-D", "warnings"]);
    cmd("cargo", &c)
}

fn run_test_lib(args: &[String]) -> Result<()> {
    let mut c = vec!["nextest", "run", "--workspace", "--exclude", "xtask"];
    c.extend(args.iter().map(|s| s.as_str()));
    cmd("cargo", &c)
}

fn run_test_doc(args: &[String]) -> Result<()> {
    let mut c = vec!["test", "--doc", "--workspace", "--exclude", "xtask"];
    c.extend(args.iter().map(|s| s.as_str()));
    cmd("cargo", &c)
}

fn run_cov(args: &[String]) -> Result<()> {
    let cwd = env::current_dir()?;
    let cwd_str = cwd.to_string_lossy();
    let llvm_profile_file = format!("{}/%m-%p.profraw", cwd_str);

    let mut build_cmd = Command::new("cargo");
    build_cmd
        .args(["build", "--workspace", "--exclude", "autd3-examples"])
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", &llvm_profile_file);
    run_cmd(build_cmd)?;

    let mut test_cmd = Command::new("cargo");
    test_cmd
        .args(["test", "--workspace", "--exclude", "autd3-examples"])
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", &llvm_profile_file);
    run_cmd(test_cmd)?;

    let t_arg = if args.is_empty() { "html" } else { &args[0] };

    let grcov_c = vec![
        ".",
        "-s",
        ".",
        "--binary-path",
        "./target/debug",
        "--llvm",
        "--branch",
        "--ignore-not-existing",
        "-o",
        "./coverage",
        "-t",
        t_arg,
        "--excl-line",
        "GRCOV_EXCL_LINE|#\\[derive|unreachable!|unimplemented!|Infallible|^\\s*\\)+\\?\\)*[,;]?$",
        "--keep-only",
        "autd3/src/**/*.rs",
        "--keep-only",
        "autd3-core/src/**/*.rs",
        "--keep-only",
        "autd3-driver/src/**/*.rs",
        "--keep-only",
        "autd3-firmware-emulator/src/**/*.rs",
        "--keep-only",
        "autd3-gain-holo/src/**/*.rs",
        "--excl-start",
        "GRCOV_EXCL_START",
        "--excl-stop",
        "GRCOV_EXCL_STOP",
    ];
    cmd("grcov", &grcov_c)?;

    for entry in std::fs::read_dir(cwd)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension()
            && ext == "profraw"
        {
            let _ = std::fs::remove_file(path);
        }
    }

    Ok(())
}

fn run_update_version(version: &str) -> Result<()> {
    use regex::Regex;
    let path = Path::new("Cargo.toml");
    let content = std::fs::read_to_string(path)?;

    let re_version = Regex::new(r#"(?m)^version = ".*?""#)?;
    let content = re_version.replace_all(&content, format!(r#"version = "{}""#, version));

    let re_autd3 = Regex::new(r#"(?m)^autd3(.*)version = ".*?""#)?;
    let content = re_autd3.replace_all(&content, format!(r#"autd3${{1}}version = "{}""#, version));

    std::fs::write(path, content.as_ref())?;
    Ok(())
}

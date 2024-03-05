use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::io;
use std::io::Write;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Install { packages: Vec<String> },
    Update,
}

fn main() {
    let args = Args::parse();

    let apt_option = [
        "-o",
        "debug::nolocking=true",
        "-o",
        "dir::cache=/tmp/apt/cache",
        "-o",
        "dir::state=/tmp/apt/state",
    ];

    let _ = fs::remove_dir_all("/tmp/apt/cache/");
    fs::create_dir_all("/tmp/apt/cache/lists/partial").expect("error creating cache");
    fs::create_dir_all("/tmp/apt/state/archives/partial").expect("error creating cache");
    let home = format!("{}/.apt", env::var("HOME").expect("HOME not defined"));
    fs::create_dir_all(home.clone()).expect("error creating cache");

    match args.command {
        Command::Install { packages } => {
            let mut cmd = std::process::Command::new("apt-get");
            cmd.arg("install");
            cmd.arg("--reinstall");
            cmd.args(apt_option);
            cmd.arg("-d");
            cmd.arg("-y");
            cmd.args(packages);
            println!("{:?}", cmd);
            let output = cmd.output().expect("failed to execute process");
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
            if output.status.success() {
                fs::read_dir("/tmp/apt/cache/archives/")
                    .unwrap()
                    .for_each(|entry| {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if !path.ends_with("dkpg") {
                            return;
                        }
                        let mut cmd = std::process::Command::new("dpkg");
                        cmd.arg("-x");
                        cmd.arg(path);
                        cmd.arg(home.clone());
                        let output = cmd.output().expect("failed to execute process");
                        io::stdout().write_all(&output.stdout).unwrap();
                        io::stderr().write_all(&output.stderr).unwrap();
                    });
            }
        }
        Command::Update => {
            let mut cmd = std::process::Command::new("apt-get");
            cmd.arg("update");
            cmd.args(apt_option);
            let output = cmd.output().expect("failed to execute process");
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
    }
}

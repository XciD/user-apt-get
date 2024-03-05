use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::process::exit;
use std::process::Stdio;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Install {
        packages: Vec<String>,
        #[arg(short, long)]
        yes: bool,
    },
    Update,
}

fn main() {
    let args = Args::parse();

    let mut apt_option = vec![
        "-o",
        "debug::nolocking=true",
        "-o",
        "dir::cache=/tmp/apt/cache",
        "-o",
        "dir::state=/tmp/apt/state",
        "-o",
        "dir::cache::archives=/tmp/apt/cache/archives",
    ];

    let _ = fs::remove_dir_all("/tmp/apt/cache/");
    fs::create_dir_all("/tmp/apt/cache/lists/partial").expect("error creating cache");
    fs::create_dir_all("/tmp/apt/state/archives/partial").expect("error creating cache");
    let home = format!("{}/.apt", env::var("HOME").expect("HOME not defined"));
    fs::create_dir_all(home.clone()).expect("error creating cache");

    match args.command {
        Command::Install { packages, yes } => {
            if yes {
                apt_option.push("-y");
            }
            let mut cmd = std::process::Command::new("apt-get")
                .arg("install")
                .arg("--reinstall")
                .args(apt_option)
                .arg("-d")
                .args(packages)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("failed to execute process");
            let status = cmd.wait().expect("failed to wait on child");
            if status.success() {
                fs::read_dir("/tmp/apt/cache/archives")
                    .expect("read dir")
                    .for_each(|entry| {
                        let entry = entry.expect("entry to exist");
                        let path = entry.path();
                        match path.extension() {
                            Some(ext) if ext == "deb" => {}
                            _ => return,
                        }
                        let mut cmd = std::process::Command::new("dpkg")
                            .arg("-x")
                            .arg(path)
                            .arg(home.clone())
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .spawn()
                            .expect("failed to execute process");
                        cmd.wait().expect("failed to wait on child");
                    });
            }
            exit(status.code().unwrap_or(1))
        }
        Command::Update => {
            let mut cmd = std::process::Command::new("apt-get")
                .arg("update")
                .args(apt_option)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("failed to execute process");
            let status = cmd.wait().expect("failed to wait on child");
            exit(status.code().unwrap_or(1))
        }
    }
}

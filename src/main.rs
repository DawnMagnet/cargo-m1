extern crate clap;
extern crate cargo;

use std::path::PathBuf;
use clap::{Arg, App, SubCommand};
use std::process::{Command, exit, Stdio};
use cargo::util::important_paths::find_root_manifest_for_wd;

const REQUIRED_TOOLCHAIN: [&str; 2] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin"
];

pub fn check_and_repair_rust_toolchain() {
    for tool_chain_name in REQUIRED_TOOLCHAIN {
        Command::new("rustup")
            .args(["target", "add", tool_chain_name])
            .output().expect("rustup command error");
    }
}

pub fn get_target_directory_path() -> PathBuf {
    let config = cargo::util::config::Config::default().unwrap();
    let res = find_root_manifest_for_wd(config.cwd());
    if let Ok(mut path) = res {
        path.pop();
        path.push("target");
        return path;
    }
    println!("Cargo Config Error! Please Check current directory");
    exit(1);
}

pub fn get_binary_path(dir: PathBuf) -> String {
    for entry in dir.as_path().read_dir().unwrap() {
        let d = entry.unwrap();
        if d.metadata().unwrap().is_dir() {
            continue;
        }
        let p = d.path();
        let last = p.file_name().unwrap().to_str().unwrap().to_string();
        if last.starts_with(".") || last.ends_with(".d") {
            continue;
        }
        return p.to_str().unwrap().to_string();
    }
    return dir.to_str().unwrap().to_string();
}

fn main() {
    let author = "DawnMagnet <a188504964@gmail.com>";
    let version = "0.1.0";
    let app_name = "cargo-m1";
    let mut vargs = vec![];
    for argument in std::env::args_os() {
        vargs.push(argument.to_str().unwrap().to_string());
    }
    if vargs[1] == "m1" {
        vargs.remove(1);
    }
    let matches = App::new(app_name)
        .version(version)
        .author(author)
        .about("Build Rust Universal Binary on Your M1-arm64 MacBook & Mac")
        .subcommand(
            SubCommand::with_name("build")
                .about("cargo-m1 build command")
                .version(version)
                .arg(Arg::with_name("release")
                    .long("release")
                    .required(false)
                )
        ).get_matches_from(vargs);
    if let (_command, Some(nxt_match)) = matches.subcommand() {
        let is_release = nxt_match.index_of("release").is_some();
        if !(cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")) {
            println!("System Not Satisfied!\\
                Please ensure the environment is MacOS with Apple Silicon.")
        }

        check_and_repair_rust_toolchain();
        let target_directory_path = get_target_directory_path();
        let mut save_directory = target_directory_path.clone();
        save_directory.pop();
        save_directory.push("universal_binary");
        let mut binary_list = vec![];
        for toolchain in REQUIRED_TOOLCHAIN {
            Command::new("cargo")
                .args(if is_release {
                    vec!["build", "--release", "--target", toolchain]
                } else {
                    vec!["build", "--target", toolchain]
                })
                .stdout(Stdio::piped())
                .spawn()
                .expect("cargo command error")
                .wait().expect("cargo command error");
            println!("{} Build Done!", toolchain);
            let mut bin_path = target_directory_path.clone();
            bin_path.push(toolchain);
            bin_path.push(if is_release {"release"} else {"debug"});
            let bin_localtion = get_binary_path(bin_path);
            binary_list.push(bin_localtion);
        }
        Command::new("lipo")
            .args(["-create", "-output"])
            .arg(save_directory.to_str().unwrap().to_string())
            .args(binary_list)
            .spawn().expect("lipo error").wait().expect("lipo error");
        println!("Already Generate Universal Binary!");
    } else {
        println!("{}", matches.usage());
        println!("Please use {} help for more information.", app_name)
    }
}

use std::fs;

use clap::Arg;
use clap::Command;
use clap_complete::{generate_to, Shell};

fn main() {
    let var = std::env::var_os("SHELL_COMPLETIONS_DIR").or_else(|| std::env::var_os("OUT_DIR"));
    let outdir = match var {
        None => return,
        Some(outdir) => outdir,
    };
    fs::create_dir_all(&outdir).unwrap();
    let mut command = Command::new("qcloud")
        .subcommand(
            Command::new("upload")
                .arg(
                    Arg::new("file-name")
                        .short('f')
                        .help("本地文件名称")
                        .required(true),
                )
                .arg(
                    Arg::new("key-name")
                        .short('k')
                        .help("对象名称, 如果未指定，和本地文件名称相同")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("download")
                .arg(
                    Arg::new("file-name")
                        .short('f')
                        .help("本地保存文件名称, 如果未指定，和对象名称相同")
                        .required(false),
                )
                .arg(
                    Arg::new("key-name")
                        .short('k')
                        .help("对象名称")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("delete").arg(
                Arg::new("key-name")
                    .short('k')
                    .help("对象名称")
                    .required(true),
            ),
        );
    for shell in [
        Shell::Bash,
        Shell::Fish,
        Shell::Zsh,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        generate_to(shell, &mut command, "qcloud", &outdir).unwrap();
    }
}

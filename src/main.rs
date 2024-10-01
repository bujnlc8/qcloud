//! 上传下载删除腾讯云对象存储(qcos)文件

use clap::{CommandFactory, Parser, Subcommand};

use clap_complete::{generate, Shell};
use colored::Colorize;
use qcos::objects::ErrNo;
use qrcode::{render::unicode, QrCode};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf, process::exit, str::FromStr, time::SystemTime};
use tokio::fs::{self, read_to_string};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    secrect_key: String,
    secrect_id: String,
    bucket_name: String,
    region: String,
    domain: Option<String>,
}

// 将列表分块
fn split_into_chunks<T>(list: Vec<T>, chunk_count: usize) -> Vec<Vec<T>>
where
    T: Clone,
{
    //let chunk_size = (list.len() + chunk_count - 1) / chunk_count;
    let chunk_size = list.len().div_ceil(chunk_count);
    list.chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

// 将配置文件读出
async fn read_to_config(path: &str) -> Config {
    toml::from_str::<Config>(
        read_to_string(path)
            .await
            .unwrap_or_else(|e| {
                eprintln!("读取配置文件{}出错, {}", path, e);
                exit(1);
            })
            .as_str(),
    )
    .unwrap_or_else(|e| {
        eprintln!("配置文件{}格式错误, {}", path, e);
        exit(1);
    })
}

// 获取配置文件路径，先从环境变量读，如果没读到，返回默认路径
fn get_config_path(path: Option<String>) -> String {
    if let Some(p) = path {
        return p;
    }
    match std::env::var("QCLOUD_CONFIG_DIR") {
        Ok(e) => e,
        Err(_) => {
            let mut c = dirs::home_dir().unwrap();
            c.push(".config/qcloud.toml");
            c.to_str().unwrap().to_string()
        }
    }
}

// 获取下载链接
fn get_download_url(
    domain: Option<String>,
    bucket_name: &str,
    key_name: &str,
    region: &str,
) -> String {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    match domain {
        Some(domain) => {
            if !domain.is_empty() {
                format!("https://{domain}/{key_name}?t={:2x}", ts)
            } else {
                format!(
                    "https://{bucket_name}.cos.{region}.myqcloud.com/{key_name}?t={:2x}",
                    ts
                )
            }
        }
        None => format!(
            "https://{bucket_name}.cos.{region}.myqcloud.com/{key_name}?t={:2x}",
            ts
        ),
    }
}

// 遍历目录
fn walk_dir(dir: PathBuf) -> Vec<PathBuf> {
    let mut res = Vec::new();
    if dir.is_dir() {
        for item in dir.read_dir().unwrap() {
            let item_path = item.unwrap().path();
            if item_path.is_file() {
                res.push(item_path);
            } else {
                res.extend(walk_dir(item_path));
            }
        }
    } else {
        res.push(dir);
    }
    res
}

#[derive(Parser)]
#[clap(author, version, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// 配置文件路径
    #[clap(short, long)]
    config: Option<String>,

    /// 生成shell补全脚本, 支持Bash, Zsh, Fish, PowerShell, Elvish
    #[arg(long)]
    completion: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// 上传文件或文件夹到腾讯云对象存储
    Upload(Upload),
    /// 从腾讯云对象存储下载文件到本地
    Download(Download),
    /// 从腾讯云存储删除文件
    Delete(Delete),
}

#[derive(clap::Args)]
struct Upload {
    /// 本地文件路径，支持上传文件夹
    #[clap(short, long)]
    file_path: PathBuf,

    /// 对象名称, 如果未指定，和文件名称相同，如果是上传文件夹，则是目录名称
    #[clap(short, long)]
    key_name: Option<String>,

    /// 最大上传线程数量，默认20
    #[clap(long)]
    max_threads: Option<u64>,

    /// 分片上传的大小，单位MB，1M-1GB之间，默认50M
    #[clap(long)]
    part_size: Option<u64>,

    /// 不显示上传进度条
    #[clap(long)]
    no_progress_bar: bool,

    /// 不输出下载链接二维码
    #[clap(long)]
    no_qrcode: bool,
}

#[derive(clap::Args)]
struct Download {
    /// 对象名称
    #[clap(short, long)]
    key_name: String,
    /// 本地保存文件名称, 如果未指定，和对象名称相同
    #[clap(short, long)]
    file_name: Option<String>,

    /// 下载线程数量，默认5
    #[clap(long)]
    threads: Option<u8>,

    /// 不显示下载进度条
    #[clap(long)]
    no_progress_bar: bool,
}

#[derive(clap::Args)]
struct Delete {
    /// 对象名称，不支持文件夹
    #[clap(short, long)]
    key_name: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = read_to_config(&get_config_path(cli.config)).await;
    if config.bucket_name.is_empty() {
        eprintln!("bucket_name为空！");
        exit(1)
    }
    let client = qcos::client::Client::new(
        &config.secrect_id,
        &config.secrect_key,
        &config.bucket_name,
        &config.region,
    );
    let start = std::time::Instant::now();
    match cli.command {
        Some(command) => match command {
            Commands::Upload(e) => {
                let file_path = e.file_path;
                let part_size = e.part_size.map(|e| e * 1024 * 1024);
                let path = std::path::Path::new(&file_path);
                if !path.exists() {
                    eprintln!("{}", "文件不存在或不可读！".red());
                    exit(1);
                }
                if path.is_dir() {
                    let item_path = walk_dir(path.into());
                    if item_path.is_empty() {
                        eprintln!("{}", "空文件夹！".red());
                        exit(1);
                    }
                    // 最多30个线程上传
                    let item_path_list = split_into_chunks(item_path, 30);
                    let dir_name = file_path
                        .clone()
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    let mut handles = vec![];
                    for item_paths in item_path_list {
                        let item_paths = item_paths.clone();
                        let file_name = file_path.clone();
                        let key_name = e.key_name.clone();
                        let client = client.clone();
                        let config = config.clone();
                        let dir_name = dir_name.clone();
                        let handle = tokio::spawn(async move {
                            let mut success = 0;
                            let mut fail = 0;
                            for item in item_paths {
                                let mut object_name = item.to_str().unwrap().to_string();
                                if let Some(ref dest_dir) = key_name {
                                    let dest_dir = dest_dir.strip_prefix("/").unwrap_or(dest_dir);
                                    let dest_dir = dest_dir.strip_suffix("/").unwrap_or(dest_dir);
                                    object_name = format!(
                                        "{dest_dir}/{dir_name}/{}",
                                        object_name
                                            .strip_prefix(file_name.to_str().unwrap())
                                            .unwrap_or(&object_name)
                                    );
                                    object_name = object_name.replace("//", "/").to_lowercase();
                                }
                                let resp = client
                                    .clone()
                                    .put_big_object(
                                        &item,
                                        &object_name,
                                        None,
                                        None,
                                        None,
                                        part_size,
                                        Some(1),
                                    )
                                    .await;
                                if resp.error_no != ErrNo::SUCCESS {
                                    eprintln!(
                                        "😭 {} -> {} 上传失败, [{}] {}",
                                        item.to_str().unwrap().green(),
                                        object_name.yellow(),
                                        resp.error_no,
                                        resp.error_message.red(),
                                    );
                                    fail += 1;
                                } else {
                                    println!(
                                        "🚀 {} -> {} 上传成功\n🔗 {}\n",
                                        item.to_str().unwrap().green(),
                                        object_name.yellow(),
                                        get_download_url(
                                            config.domain.clone(),
                                            &config.bucket_name,
                                            &object_name,
                                            &config.region
                                        )
                                    );
                                    success += 1;
                                }
                            }
                            (success, fail)
                        });
                        handles.push(handle);
                    }
                    let mut success = 0;
                    let mut fail = 0;
                    for handle in handles {
                        let res = handle.await.unwrap();
                        success += res.0;
                        fail += res.1;
                    }
                    println!(
                        "🚀 文件夹 {} 上传完成\n🔥 {} 个文件上传成功, {} 个文件上传失败, {:.2}s elapsed.",
                        path.to_str().unwrap().green(),
                        success.to_string().green(),
                        fail.to_string().red(),
                        start.elapsed().as_secs_f64(),
                    );
                    return;
                }
                let mut key_name = match e.key_name {
                    Some(key_name) => key_name,
                    None => format!(
                        "uploads/{}",
                        file_path.file_name().unwrap().to_str().unwrap()
                    ),
                };
                key_name = key_name.replace("//", "/");
                let resp = if !e.no_progress_bar {
                    client
                        .put_big_object_progress_bar(
                            &file_path,
                            &key_name,
                            None,
                            None,
                            None,
                            part_size,
                            e.max_threads,
                            None,
                        )
                        .await
                } else {
                    client
                        .put_big_object(
                            &file_path,
                            &key_name,
                            None,
                            None,
                            None,
                            part_size,
                            e.max_threads,
                        )
                        .await
                };
                if resp.error_no != ErrNo::SUCCESS {
                    eprintln!(
                        "😭 {} -> {} 上传失败, [{}] {}",
                        file_path.to_str().unwrap().green(),
                        key_name.yellow(),
                        resp.error_no,
                        resp.error_message.red(),
                    );
                } else {
                    println!(
                        "🚀 {} -> {} 上传成功, {:.2}s elapsed.",
                        file_path.to_str().unwrap().green(),
                        key_name.yellow(),
                        start.elapsed().as_secs_f64()
                    );
                    let download_url = get_download_url(
                        config.domain.clone(),
                        &config.bucket_name,
                        &key_name,
                        &config.region,
                    );
                    println!("🔗 {}", download_url.yellow());
                    if !e.no_qrcode {
                        let code = QrCode::new(download_url).unwrap();
                        let image = code
                            .render::<unicode::Dense1x2>()
                            .module_dimensions(1, 1)
                            .dark_color(unicode::Dense1x2::Light)
                            .light_color(unicode::Dense1x2::Dark)
                            .build();
                        println!("{}", image);
                    }
                }
            }
            Commands::Download(e) => {
                let key_name = e.key_name;
                let file_name = match e.file_name {
                    Some(file_name) => file_name,
                    None => key_name
                        .split("/")
                        .last()
                        .unwrap_or(key_name.as_str())
                        .to_string(),
                };
                let path = PathBuf::from(&file_name);
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).await.unwrap();
                    }
                }
                let resp = if e.no_progress_bar {
                    client.get_object(&key_name, &file_name, e.threads).await
                } else {
                    client
                        .get_object_progress_bar(&key_name, &file_name, e.threads, None)
                        .await
                };
                if resp.error_no != ErrNo::SUCCESS {
                    eprintln!("{}", format!("😭 下载失败, {}", resp.error_message).red());
                } else {
                    println!(
                        "🚀 下载成功 {}, {:.2}s elapsed.",
                        file_name.green(),
                        start.elapsed().as_secs_f64()
                    );
                }
            }
            Commands::Delete(e) => {
                let resp = client.delete_object(&e.key_name).await;
                if resp.error_no != ErrNo::SUCCESS {
                    eprintln!(
                        "😭 删除失败 {}, [{}] {}",
                        e.key_name.green(),
                        resp.error_no,
                        resp.error_message.red(),
                    );
                } else {
                    println!(
                        "🚀 删除成功 {}, {:.2}mss elapsed.",
                        e.key_name.green(),
                        start.elapsed().as_millis()
                    );
                }
            }
        },
        None => match cli.completion {
            Some(shell) => {
                let mut cmd = Cli::command();
                let bin_name = cmd.get_name().to_string();
                match Shell::from_str(&shell.to_lowercase()) {
                    Ok(shell) => generate(shell, &mut cmd, bin_name, &mut io::stdout()),
                    Err(e) => {
                        eprintln!("😭 生成补全脚本失败, {}", e.red());
                        exit(1)
                    }
                };
            }
            None => {
                Cli::command().print_help().unwrap();
            }
        },
    }
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use crate::{get_config_path, read_to_config, split_into_chunks, walk_dir};

    #[tokio::test]
    async fn test_read_to_config() {
        let res = read_to_config("qcloud.toml.example").await;
        assert_eq!(res.region, "region");
        assert_eq!(res.bucket_name, "bucket_name");
        assert_eq!(res.secrect_key, "foo");
        assert_eq!(res.secrect_id, "bar");
        assert!(res.domain.is_some_and(|x| x == "static.example.com"));
    }

    #[test]
    fn test_get_config_path() {
        println!("{}", get_config_path(None));
    }

    #[test]
    fn test_walk_dir() {
        let res = walk_dir(PathBuf::from(".git"));
        println!("{:#?}", res);
    }

    #[test]
    fn test_split_into_chunks() {
        let list = vec![1, 2, 3, 4, 5];
        let chunk_count = 10;

        let chunks = split_into_chunks(list, chunk_count);

        for (i, chunk) in chunks.iter().enumerate() {
            println!("Chunk {}: {:?}", i, chunk);
        }
    }
}

use clap::{Parser, Subcommand};

use qcos::objects::{mime, ErrNo, Objects};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    secrect_key: String,
    secrect_id: String,
    bucket_name: String,
    region: String,
}

impl Config {
    fn blank_config() -> Config {
        Config {
            secrect_id: "".to_string(),
            secrect_key: "".to_string(),
            bucket_name: "".to_string(),
            region: "".to_string(),
        }
    }
}

// 将配置文件读出
fn read_to_config(path: &str) -> Config {
    match read_to_string(path) {
        Ok(e) => match toml::from_str::<Config>(e.as_str()) {
            Ok(e) => e,
            Err(e) => {
                println!("配置文件格式错误，{}", e);
                Config::blank_config()
            }
        },
        Err(e) => {
            println!("读取配置文件{}出错, {}", path, e);
            Config::blank_config()
        }
    }
}

// 获取配置文件路径，先从环境变量读，如果没读到，返回默认路径
fn get_config_path() -> String {
    match std::env::var("QCLOUD_CONFIG_DIR") {
        Ok(e) => e,
        Err(_) => {
            let mut c = dirs::home_dir().unwrap();
            c.push(".config/qcloud.toml");
            c.to_str().unwrap().to_string()
        }
    }
}

#[derive(Parser)]
#[clap(author, version, long_about = None)]
#[clap(about = "操作腾讯云对象存储")]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 上传文件到腾讯云对象存储
    Upload(Upload),
    /// 从腾讯云对象存储下载文件到本地
    Download(Download),
    /// 从腾讯云存储删除文件
    Delete(Delete),
}

#[derive(clap::Args)]
struct Upload {
    /// 本地文件名称
    #[clap(short, long)]
    file_name: String,
    /// 对象名称, 如果未指定，和本地文件名称相同
    #[clap(short, long)]
    key_name: Option<String>,
}

#[derive(clap::Args)]
struct Download {
    /// 对象名称
    #[clap(short, long)]
    key_name: String,
    /// 本地保存文件名称, 如果未指定，和对象名称相同
    #[clap(short, long)]
    file_name: Option<String>,
}

#[derive(clap::Args)]
struct Delete {
    /// 对象名称
    #[clap(short, long)]
    key_name: String,
}

fn main() {
    let path = get_config_path();
    let config = read_to_config(path.as_str());
    if config.bucket_name.is_empty() {
        return;
    }
    let client = qcos::client::Client::new(
        config.secrect_id.as_str(),
        config.secrect_key.as_str(),
        config.bucket_name.as_str(),
        config.region.as_str(),
    );
    match Cli::parse().command {
        Commands::Upload(e) => {
            let file_name = e.file_name;
            let mut key_name = &file_name;
            if let Some(ref key) = e.key_name {
                key_name = key;
            }
            let path = std::path::Path::new(&file_name);
            if path.is_dir() {
                println!("不能上传文件夹");
                return;
            }
            if !path.exists() {
                println!("文件不存在或不可读");
                return;
            }
            match std::fs::File::open(file_name.as_str()) {
                Ok(file) => {
                    let mut me = mime::APPLICATION_OCTET_STREAM;
                    let guess = mime_guess::from_path(file_name.as_str());
                    if let Some(e) = guess.first() {
                        me = e;
                    }
                    let resp = client.put_object_binary(file, key_name, me, None);
                    if resp.error_no != ErrNo::SUCCESS {
                        println!("上传失败, {}", resp.error_message);
                    } else {
                        println!("上传成功");
                    }
                }
                Err(e) => {
                    println!("打开文件失败, {}", e);
                }
            }
        }
        Commands::Download(e) => {
            let key_name = e.key_name;
            let mut file_name = &key_name;
            if let Some(ref file) = e.file_name {
                file_name = file;
            }
            let resp = client.get_object(key_name.as_str(), file_name);
            if resp.error_no != ErrNo::SUCCESS {
                println!("下载失败, {}", resp.error_message);
            } else {
                println!("下载成功");
            }
        }
        Commands::Delete(e) => {
            let key_name = e.key_name;
            let resp = client.delete_object(key_name.as_str());
            if resp.error_no != ErrNo::SUCCESS {
                println!("删除失败, {}", resp.error_message);
            } else {
                println!("删除成功")
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{get_config_path, read_to_config};

    #[test]
    fn test_read_to_config() {
        let res = read_to_config("qcloud.toml.example");
        assert_eq!(res.region, "region");
        assert_eq!(res.bucket_name, "bucket_name");
        assert_eq!(res.secrect_key, "foo");
        assert_eq!(res.secrect_id, "bar");
    }

    #[test]
    fn test_get_config_path() {
        println!("{}", get_config_path());
    }
}

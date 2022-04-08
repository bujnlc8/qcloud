[![qcloud](https://github.com/bujnlc8/qcloud/actions/workflows/qcloud.yml/badge.svg?branch=master)](https://github.com/bujnlc8/qcloud/actions/workflows/qcloud.yml)

一个操作腾讯云对象存储的命令行工具。

# 安装

```sh
cargo install qcloud
```

# 配置

```sh

# 替换成你的配置
1. mv qcloud.toml.example qcloud.toml 

# 将qcloud.toml移动到 $HOME/.config 目录下，也可以通过`QCLOUD_CONFIG_DIR`环境变量来定义配置文件所在的目录
2. mv qcloud.toml $HOME/.config 

```

# 使用

``` sh

# 上传文件
1. qcloud upload -f 本地文件 [-k 对象key-name]  

# 下载文件
2. qcloud download -k 对象key-name [-f 本地保存文件名称] 

```

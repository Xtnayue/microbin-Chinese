
![Screenshot](.github/index.png)

# MicroBin（简体中文版）

![Build](https://github.com/szabodanika/microbin/actions/workflows/rust.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/microbin.svg)](https://crates.io/crates/microbin)
[![Docker Image](https://github.com/szabodanika/microbin/actions/workflows/release.yml/badge.svg)](https://hub.docker.com/r/danielszabo99/microbin)
[![Docker Pulls](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)](https://img.shields.io/docker/pulls/danielszabo99/microbin?label=Docker%20pulls)

MicroBin 是一个超小、功能丰富、可配置、自包含且可自托管的粘贴板（paste bin）Web 应用。它极易部署和使用，仅需几 MB 内存与磁盘空间，几分钟即可跑起来，不妨现在就试试？

> 英文文档见 [README.en.md](README.en.md)。

## 关于简体中文版

本仓库是 [microbin](https://github.com/szabodanika/microbin) 的中文本地化分支，在保留上游功能的前提下提供**中英双语界面**。

### 语言切换

界面语言在启动时确定，支持命令行参数与环境变量两种方式：

```bash
# 方式一：命令行参数
microbin --language zh

# 方式二：环境变量
export MICROBIN_LANGUAGE=zh
microbin
```

取值说明：

| 取值 | 效果 |
| --- | --- |
| `en`（默认） | 英文界面，与上游完全一致 |
| `zh` | 简体中文界面 |

同时支持 `zh-cn`、`zh_cn`、`cn`、`zh-hans` 等写法，均等价于 `zh`；无法识别的取值会回退为英文。

### 汉化实现方式

- 翻译表位于 [`src/i18n.rs`](src/i18n.rs)，以**英文原文作为键**、中文译文作为值。因此英文环境是恒等映射，模板中直接书写英文原文即可，可读性最好；中文环境查表替换，未命中的条目自动回退英文。
- 模板中通过自定义 Askama 过滤器调用：`{{ "New"|t }}`。
- Rust 代码中通过 `crate::i18n::tr("...")` 调用。
- 未纳入翻译的内容：品牌名（MicroBin）、编程语言名（`Rust`、`Python` 等语法高亮选项）、环境变量名与配置项标识（管理页中的 `editable`、`port` 等）、日志输出。

欢迎继续补充翻译：在 `src/i18n.rs` 的 `ZH` 表中增加条目即可。

### 上游功能改动

除界面文案外，本分支不修改上游的任何业务逻辑，仅新增了 `--language` 参数与 `src/i18n.rs`。

### 本地构建（Windows）

默认 feature 会**从源码编译 OpenSSL**（`openssl` 的 `vendored`）、**SQLite**（`rusqlite` 的 `bundled`）、**oniguruma** 与 **zstd**，所以需要一个可用的 C 工具链。

#### 推荐：MSVC 工具链（完整功能）

需要三样东西：

1. **Visual Studio 生成工具**（勾选「使用 C++ 的桌面开发」，提供 `cl.exe` / `link.exe`）与 **Windows SDK**
2. **Strawberry Perl** —— OpenSSL 的 `Configure` 依赖完整的 Perl。**Git for Windows 自带的 perl 是被裁剪过的，会报 `Can't locate Locale/Maketext/Simple.pm`，不能用。**
3. **NASM** —— OpenSSL 在 Windows 上启用汇编优化时需要

```bash
winget install Microsoft.VisualStudio.2022.BuildTools
winget install StrawberryPerl.StrawberryPerl
winget install NASM.NASM
```

装好后**不需要手动执行 `vcvars64.bat`**：`rustc` 通过注册表定位 MSVC 链接器，`cc` 通过 vswhere 定位 `cl.exe` 并自动注入 `INCLUDE` / `LIB`。

把 Strawberry Perl 放在 `PATH` **最前面**（压过 Git 自带的 perl），Git 的 `usr/bin` 放**最后**（它里面的 `link.exe` 是个文件工具，会遮蔽 MSVC 的链接器）：

```bash
export PATH="/c/Strawberry/perl/bin:/c/Program Files/NASM:$PATH:/c/Program Files/Git/usr/bin"
cargo build --release
```

产物为 `target/release/microbin.exe`：

```bash
./target/release/microbin.exe --language zh
```

数据库默认走 SQLite，位于 `microbin_data/database.sqlite`。

> 首次构建要从源码编译 OpenSSL，约 **25–30 分钟**；之后的增量构建约 2 分钟。

#### 备选：没有 MSVC 时改用 MinGW

若本机没有 MSVC 生成工具（缺少 `link.exe` 与 Windows SDK），默认的 `x86_64-pc-windows-msvc` 工具链无法链接。此时可改用 MinGW 工具链：

```bash
# 1. 安装 MinGW-w64
winget install BrechtSanders.WinLibs.POSIX.UCRT

# 2. 安装 gnu 宿主工具链
#    注意：只加 --target 是不够的——Cargo 的构建脚本与 proc-macro 永远按
#    宿主平台编译，必须换成 gnu 宿主的工具链
rustup toolchain install stable-x86_64-pc-windows-gnu

# 3. 构建（--no-default-features 可避开 openssl/oniguruma 的 C 依赖）
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=gcc
export CC=gcc
cargo +stable-x86_64-pc-windows-gnu build --release \
    --no-default-features --features no-c-deps
```

> ⚠️ `--no-default-features` 会**关闭 SQLite 支持**，运行时必须加 `--json-db` 改用 JSON 后端：
> `./target/release/microbin.exe --json-db --language zh`
>
> 另外 JSON 后端的库路径固定为 `pasta_data/database.json`，**不随 `--data-dir` 变化**，启动前需保证该目录存在（`mkdir -p pasta_data`），否则启动即 panic。

## 自行部署 MicroBin

运行我们的快速 Docker 部署脚本（[DockerHub](https://hub.docker.com/r/danielszabo99/microbin)）：

```bash
bash <(curl -s https://microbin.eu/docker.sh)
```

或从 [Cargo](https://crates.io/crates/microbin) 手动安装：

```bash
cargo install microbin;
curl -L -O https://raw.githubusercontent.com/szabodanika/microbin/master/.env;
source .env;
microbin
```

在官网 [microbin.eu](https://microbin.eu) 上你可以找到：

- [界面截图](https://microbin.eu/screenshots/)
- [指南与文档](https://microbin.eu/docs/intro)
- [路线图](https://microbin.eu/roadmap)

## 功能特性

- 完全自包含的可执行文件，MicroBin 只有一个文件！
- 服务端与客户端端到端加密
- 文件上传（例如 `server.com/file/pig-dog-cat`）
- 原始文本直出（例如 `server.com/raw/pig-dog-cat`）
- 二维码支持
- 短链接与重定向
- 用动物名称代替随机数字作为上传标识（共 64 种动物）
- 支持多个附件
- 支持 SQLite 与 JSON 两种数据库
- 上传内容可设为公开或私有、可编辑或不可编辑、自动过期或永不过期
- 自动深色模式，仅用极少量 CSS 与原生 JS 即可自定义样式（见 [`water.css`](https://github.com/kognise/water.css)）
- 以及更多！

## 什么是「上传内容」？

在 MicroBin 中，一次上传可以是：

- 你想从一台机器粘贴到另一台机器的文本，例如一段代码；
- 你想分享的文件，例如大到 Discord 传不动的视频、代码项目的 zip 包或一张图片；
- 一个 URL 跳转。

## MicroBin 适合什么场景？

你可以用 MicroBin：

- 把长文本发给别人；
- 把大文件发给别人；
- 安全地分享机密或敏感文档；
- 当作短链接／跳转服务；
- 在 Web 上直接提供内容，例如测试用配置文件、图片，或通过 Raw 接口提供任意文件内容；
- 在桌面与只能通过命令行访问的服务器之间传文件；
- 当作「收件箱」服务，别人可以上传文件或文本给你，但看不到也删不掉其他人发的内容；
- 甚至只是用来快速记笔记。

……还有更多玩法，不妨发挥创意？

MicroBin 与 MicroBin.eu 基于 [BSD 3-Clause 许可证](LICENSE) 发布。

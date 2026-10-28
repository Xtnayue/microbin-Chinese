
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

界面语言**按访问者逐个请求协商**，不需要重启，也不需要为不同用户部署多份实例。

优先级从高到低：

| 优先级 | 来源 | 说明 |
| --- | --- | --- |
| 1 | `?lang=zh` 查询参数 | 一次性生效，不落 cookie。适合分享「指定语言的链接」 |
| 2 | `microbin_lang` Cookie | 点导航栏右侧的「中文 / English」即写入，长期有效 |
| 3 | `Accept-Language` 请求头 | 浏览器语言设置。按 q 值排序，跳过不支持的语言再取下一个（`fr,zh;q=0.9` 会落到中文） |
| 4 | `--language` / `MICROBIN_LANGUAGE` | **兜底值**，仅当访问者以上三项都没有表达偏好时使用 |

```bash
# 设置全站兜底语言（命令行参数）
microbin --language zh

# 或环境变量
export MICROBIN_LANGUAGE=zh
microbin
```

取值说明：

| 取值 | 效果 |
| --- | --- |
| `en`（默认） | 英文界面，与上游完全一致 |
| `zh` | 简体中文界面 |

同时支持 `zh-cn`、`zh_cn`、`cn`、`zh-hans` 等写法，均等价于 `zh`；无法识别的取值会回退为 `en`。

> 例外：`microbin --help` 的输出没有请求上下文，只用 `--language` 这一处的值来决定语言。

实现上，语言在中间件里解析后存入 `tokio::task_local!`，随请求 future 一起传递。不能用 thread-local——actix 每个 worker 线程会交错轮询多个请求，同一线程上的并发请求会互相覆盖语言。

### 汉化实现方式

- 翻译表位于 [`src/i18n.rs`](src/i18n.rs)，以**英文原文作为键**、中文译文作为值。因此英文环境是恒等映射，模板中直接书写英文原文即可，可读性最好；中文环境查表替换，未命中的条目自动回退英文。
- 模板中通过自定义 Askama 过滤器调用：`{{ "New"|t }}`。
- Rust 代码中通过 `crate::i18n::tr("...")` 调用。
- 未纳入翻译的内容：品牌名（MicroBin）、编程语言名（`Rust`、`Python` 等语法高亮选项）、环境变量名与配置项标识（管理页中的 `editable`、`port` 等）、日志输出。

欢迎继续补充翻译：在 `src/i18n.rs` 的 `ZH` 表中增加条目即可。

### 相对上游的功能改动

本分支基于上游 [`szabodanika/microbin`](https://github.com/szabodanika/microbin) v2.1.4，做了以下改动。

#### 一、中英双语界面

新增 `--language` 参数与 `src/i18n.rs` 翻译表，模板与服务端提示均支持 `en` / `zh`。语言**按请求协商**，`--language` 只是兜底值，访问者可用 `?lang=` / `microbin_lang` Cookie / `Accept-Language` 各自覆盖；`--help` 的输出只用 `--language` 决定。详见上文「关于简体中文版」。

#### 二、权限模型重构

废弃原先固定搭配的五档「隐私级别」（`public` / `unlisted` / `readonly` / `private` / `secret`），改为三项**相互独立、可单独授予**的权限项：

| 权限 | 取值 | 含义 | 作用范围 |
| --- | --- | --- | --- |
| `read` | `allow` / `password` | 谁能看到内容。取 `password` 时正文与全部附件在服务端以该密码加密存储 | `/upload/{id}`、`/raw/{id}`、`/file/{id}`、`/archive/{id}`、URL 短链 `/url/{id}` 与 `/u/{id}`、编辑页 |
| `write` | `allow` / `password` / `refuse` | 谁能修改或删除。`password` 只做授权判定，**不参与加密**；`refuse` 一律拒绝（管理员密码仍可删除） | 保存编辑、`/remove/{id}`、页面上「编辑 / 删除」入口的显示 |
| `list` | `yes` / `no` | 是否出现在公开的 `/list` 列表页 | 仅 `/list` |

同时**去掉了浏览器端加密**（原 `secret` 档），只保留服务端加密。

配置模板已同步跟进：`.env` 与 `compose.yaml` 里 5 个随旧模型一起失效的变量（`MICROBIN_PRIVATE`、`MICROBIN_ENABLE_READONLY`、`MICROBIN_ENCRYPTION_CLIENT_SIDE`、`MICROBIN_ENCRYPTION_SERVER_SIDE`、`MICROBIN_DEFAULT_PRIVACY`）已删除，并补上 `MICROBIN_ENABLE_UNLISTED`、`MICROBIN_ENABLE_READ_PASSWORD`、`MICROBIN_ENABLE_WRITE_PASSWORD`、`MICROBIN_DEFAULT_PERMISSIONS` 与 `MICROBIN_LANGUAGE`。这两份模板正是 `docker-setup.sh` 实际下载并使用的配置入口，若不同步，用户照文档设了变量却完全不起作用（且不会有任何报错）。

权限密码不落库：落库的是校验值 `encrypt(上传 id, 密码)`；校验时用提交上来的密码解密该值，结果恰好等于该上传的 id 即视为通过。这样做的两个好处：

1. 与正文是否为空无关 —— 修复了原模型下「纯附件上传（正文为空）无法校验密码」的问题；
2. 校验开销与内容大小无关，不必真的去解密正文。

read 密码与 write 密码职责分离：read 密码额外用于加解密正文与附件，write 密码只回答「是否允许写入」。二者彼此独立，可以相同也可以不同。详见 `/guide` 页面（`#read`、`#write`、`#list`、`#password` 四个锚点）。

相关的命令行参数也随之调整：

- 移除 `--private`、`--enable-readonly`、`--default-privacy`、`--encryption-client-side`、`--encryption-server-side`
- 新增 `--enable-unlisted`、`--enable-read-password`、`--enable-write-password`
- 新增 `--default-permissions=read=allow,write=allow,list=yes`（逗号分隔的 `键=值`，键为 `read` / `write` / `list`）
- 激活 `--default-view=gallery|list|stream`（该参数上游就有，但模板把它写死了，因此从未生效；见下文「顺带修复的上游问题」）

> 三个 `--enable-*` **既是界面开关，也是服务端策略**。关掉某一项后，新建表单不再渲染该选项；同时，绕过界面直接向 `/upload` 提交这个被禁用的取值会被 **HTTP 400** 拒绝，并返回一句可读的说明。三个开关默认都是 `true`，此时守卫一次都不会触发，行为与从前完全一致。
>
> 为什么是「拒绝」而不是上游那种「静默降级」：降级会把三种情况全部变成**比用户预期更开放**的结果 —— 提交 `read=password` 被降级成 `allow`，用户以为正文已加密，实际是明文落库；提交 `write=password` 被降级成 `allow`，用户以为别人改不了，实际谁都能改；提交 `list=no` 被降级成 `list=yes`，用户以为没有公开列出，实际已经列在 `/list` 上。三者严重程度不同，但方向一致：都是「用户设想的保护没有生效」。上游 `--enable-readonly` 正是这种写法（关掉之后提交 `readonly`，得到的反而是公开可写的上传）。拒绝则把三者统一成同一个结果 —— 上传不会创建，客户端收到 400 和一句可读的说明，不存在「以为保护了其实没有」的状态。
>
> 另有一条启动自检：若 `--default-permissions` 指定的默认档位恰好落在被关掉的选项上（例如 `read=password` 配 `--enable-read-password=false`），表单不会渲染那个选项，浏览器会静默回退到同组第一项。程序**不修改行为**，只在启动日志里打一条 `WARN`，说明实际会生效的档位。

**旧数据无需手工处理**，两个后端都会在读取时自动迁移，且**幂等**（重复启动不会二次迁移，也不会改动已迁移的数据）：

- **SQLite**（默认后端）：启动时用 `ALTER TABLE ... ADD COLUMN` 补齐新列，再按旧档位映射；
- **JSON**（`--json-db`）：`load_from_file()` 先按通用 JSON 读入，逐条识别并补齐权限字段后再反序列化。

> ⚠️ 旧版的 JSON 库路径是**硬编码**的 `pasta_data/database.json`，**不跟随 `--data-dir`**。本版本改为跟随 `--data-dir`，并做**一次性搬迁**：若新位置还没有库文件、而 `pasta_data/database.json` 存在，会自动把它移过去并打一条 warn。因此用 `--json-db` 的老用户升级后**不会看到空库**。反过来，新位置已有库文件时**绝不搬动**旧文件，避免覆盖。

映射规则如下（注意旧代码里「写入需要密码」的判定条件是 `read_only || encrypt_server`，**不是单看 `read_only`**）：

| 旧字段 | 映射到的新权限 |
| --- | --- |
| `private` | `list = no`（否则 `yes`） |
| `encrypt_server` | `read = password`（否则 `allow`） |
| `read_only` \|\| `encrypt_server` | `write = password`（否则 `allow`） |
| `encrypted_key` | `read_key` 与 `write_key`（格式相同，直接沿用，因此旧密码仍然有效） |

旧模型没有 `refuse` 这一档，所以迁移不会产生 `write = refuse`。发生迁移时两个后端都会打一条 warn 日志。

注意旧的浏览器端加密上传（原 `secret` 档）无法再在浏览器里解密，其正文将保持密文。

#### 三、页面宽度自适应

上游把 `<body>` 的宽度写死成 `max-width: 800px`（`--wide` 时 1080px），窗口再宽，内容也只是中间一条窄栏；管理页那种多列表格因此被挤成很长的竖条。现在默认跟随窗口宽度自适应，只在超过 1600px 时居中停下，避免超宽屏上一行文字过长；`--wide` 则完全占满窗口。窄屏（手机）行为不变。

#### 四、顺带修复的上游问题

重构权限模型的过程中发现并修掉了五个上游遗留问题：

1. **JSON 后端忽略 `--data-dir`** —— 上游把数据库路径硬编码成 `pasta_data/database.json`，因此 `--json-db` 下 `--data-dir` 完全不生效；而且只要 `pasta_data/` 不存在，启动阶段就会在创建临时文件时 panic（Docker 镜像里只建了 `microbin_data/`，所以容器中 `--json-db` 其实是启动即挂）。现已改为与 SQLite 后端一致，跟随 `--data-dir`，并对旧位置做一次性搬迁（见上文）。
2. **编辑受密码保护的上传会把明文写回数据库** —— 上游 `post_edit_private` 用解密后的正文覆盖了存储值，导致该上传在首次编辑之后就不再受密码保护。现改为始终用 read 密码重新加密后再落库。
3. **打包下载受保护的上传会得到无法解密的密文** —— `/archive/{id}` 原先把 `.enc` 密文直接塞进 zip；在去掉浏览器端解密之后这些文件已无意义，现改为校验 read 密码后返回**解密**的内容。
4. **URL 短链不校验读权限，受保护时静默失效** —— `/url/{id}` 与 `/u/{id}` 直接把 `content` 作为 `Location` 返回。`read=password` 时 `content` 是密文，于是浏览器被跳到一段密文上，链接坏掉（不是明文泄露）。现改为先引导到密码页 `/auth_url/{id}`，校验通过后解密目标地址再跳转；`read=allow` 的短链行为不变。
5. **`--default-view` 是个死参数** —— 该参数在上游就存在（`args.rs` 里有定义、`--help` 里也能看到），但 `templates/upload.html` 把兜底值硬编码成了 `'gallery'`，全仓没有任何一处读它，因此设了等于没设。现已改为由服务端取值渲染进页面脚本，并对取值做**白名单归一化**（只接受 `stream` / `list` / `gallery`，其余一律回退 `gallery`）。白名单不是可选项：该值会被直接插进 JS 的字符串字面量（该模板声明了 `escape = "none"`），若原样透传，`--default-view "x'; alert(1)//"` 足以破坏整页脚本。URL 上的 `?view=` 仍然优先于该默认值。

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
> JSON 后端的库文件是 `<data-dir>/database.json`，与 SQLite 后端的 `<data-dir>/database.sqlite` 一致，同样跟随 `--data-dir`；数据目录不存在时会在启动阶段自动创建。

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
- 服务端加密：`read` 权限要求密码时，正文与附件都以该密码加密存储
- 文件上传（例如 `server.com/file/pig-dog-cat`）
- 原始文本直出（例如 `server.com/raw/pig-dog-cat`）
- 二维码支持
- 短链接与重定向
- 用动物名称代替随机数字作为上传标识（共 64 种动物）
- 支持多个附件
- 支持 SQLite 与 JSON 两种数据库
- 三项相互独立的权限（`read` / `write` / `list`），可自由组合
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

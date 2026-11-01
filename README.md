![Screenshot](.github/index.png)

# MicroBin（简体中文版）

[![Rust](https://github.com/Xtnayue/microbin-Chinese/actions/workflows/rust.yml/badge.svg)](https://github.com/Xtnayue/microbin-Chinese/actions/workflows/rust.yml)
[![GHCR](https://github.com/Xtnayue/microbin-Chinese/actions/workflows/ghcr.yml/badge.svg)](https://github.com/Xtnayue/microbin-Chinese/actions/workflows/ghcr.yml)

MicroBin 是一个超小、功能丰富、可自托管的粘贴板（paste bin）Web 应用：一个可执行文件，几 MB 内存，几分钟就能跑起来。

本仓库是 [szabodanika/microbin](https://github.com/szabodanika/microbin) v2.1.4 的**简体中文分支**，提供中英双语界面，并把权限模型重构为三项相互独立的开关。

> English documentation: [README.en.md](README.en.md)

## 快速开始

一条命令拉起 Docker 部署（脚本来自本仓库，会自动下载配套的 `.env` 与 `compose.yaml`）：

```bash
bash <(curl -s https://raw.githubusercontent.com/Xtnayue/microbin-Chinese/master/docker-setup.sh)
```

脚本会依次问你三件事：安装目录（默认 `/usr/share/microbin`）、对外访问地址、监听端口（默认 `8080`）。重复执行是安全的，已存在的 `.env` 不会被覆盖。

也可以手动部署：

```bash
docker run -d --name microbin -p 8080:8080 \
  -v microbin_data:/app/microbin_data \
  ghcr.io/xtnayue/microbin-chinese:latest
```

镜像地址：`ghcr.io/xtnayue/microbin-chinese`，标签 `latest` / `master` / `sha-xxxxxxx`。

## 界面语言

语言**按访问者逐个请求协商**，切换不需要重启，也不需要为不同用户部署多份实例。导航栏右侧点「中文 / English」即可，选择会记住。

优先级：`?lang=zh` 查询参数 → `microbin_lang` Cookie → 浏览器 `Accept-Language` → `--language` 兜底值。

`--language` / `MICROBIN_LANGUAGE` 只是兜底：它决定「访问者没有任何偏好时显示什么」，不限制别人切换。取值 `en`（默认）或 `zh`，`zh-cn`、`cn`、`zh-hans` 等写法等价。

## 三项权限

废弃上游固定搭配的五档「隐私级别」，改为三个**可以自由组合**的独立开关：

| 权限 | 取值 | 含义 |
| --- | --- | --- |
| `read` | `allow` / `password` | 谁能看到内容。设为 `password` 时，正文与全部附件在服务端用该密码加密存储 |
| `write` | `allow` / `password` / `refuse` | 谁能修改或删除。`password` 只做授权判定，不加密；`refuse` 一律拒绝（管理员仍可删除） |
| `list` | `yes` / `no` | 是否出现在公开的 `/list` 列表页。直链不受影响 |

密码本身不落库，只保存一个由「上传标识 + 密码」算出的校验值，因此校验开销与内容大小无关，纯附件上传（正文为空）也能正常校验。

配套参数：`--default-permissions=read=allow,write=allow,list=yes` 设置新建时的默认组合；三个 `--enable-*` 开关决定是否提供对应的选项（关掉之后，绕过界面直接提交该取值会被 400 拒绝，而不是静默降级成更开放的行为）。

**升级无需手工处理**：SQLite 与 JSON 两个后端都会在读取旧数据时自动迁移，且幂等。旧的浏览器端加密（原 `secret` 档）已移除，这类上传的正文将保持密文。

详细说明见实例上的 `/guide` 页面。

## 其他改动

- **页面宽度**默认铺满窗口（上游写死 800px）。需要限宽用 `--max-width=1200`，或 `--wide=false` 退回 1600px 居中栏。
- **相对时间**按语言拼接，中文显示「14秒前」而不是「14 s ago」。
- 修复了上游若干问题：编辑受保护的上传会把明文写回数据库、打包下载受保护上传得到无法解密的密文、URL 短链不校验读权限、JSON 后端忽略 `--data-dir`、`--default-view` 从未生效。

## 功能特性

- 完全自包含的单个可执行文件
- 服务端加密（`read=password` 时正文与附件以该密码加密存储）
- 文件上传、多附件、原始文本直出、二维码、短链接与重定向
- 用动物名称代替随机数字作为标识
- SQLite 与 JSON 两种数据库后端
- 自动深色模式，极少量的 CSS 与原生 JS

## 文档

- 实例内置 `/guide` 页面：权限组合、密码机制、常见用法
- 管理页 `/admin`：当前生效的全部配置项与中文说明
- 上游官网 [microbin.eu](https://microbin.eu)：截图、路线图

## 许可证

基于 [BSD 3-Clause 许可证](LICENSE) 发布。

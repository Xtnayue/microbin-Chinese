#!/usr/bin/env bash
#
# MicroBin（简体中文版）Docker 一键部署脚本
#
# 用法：
#   bash <(curl -s https://raw.githubusercontent.com/Xtnayue/microbin-Chinese/master/docker-setup.sh)
#
# 脚本会拉取本仓库的 .env 与 compose.yaml（不是上游的），生成安装目录后
# 用 docker compose 拉起服务。重复执行是安全的：已存在的 .env 会保留，
# 只更新你这次填的两个值。
set -euo pipefail

REPO="Xtnayue/microbin-Chinese"
BRANCH="master"
RAW="https://raw.githubusercontent.com/${REPO}/${BRANCH}"

# `bash <(curl ...)` 会把 stdin 留给终端，交互才可用；
# 若有人用 `curl ... | bash`，read 会去读脚本本身，必须提前拦下来。
if [ ! -t 0 ]; then
    echo "请改用： bash <(curl -s ${RAW}/docker-setup.sh)" >&2
    exit 1
fi

# wget 优先，没有就用 curl；两者都没有就直接报错，不要等到下载时才炸
if command -v wget >/dev/null 2>&1; then
    fetch() { wget -q -O "$2" "$1"; }
elif command -v curl >/dev/null 2>&1; then
    fetch() { curl -fsSL -o "$2" "$1"; }
else
    echo "需要 wget 或 curl 之一，请先安装。" >&2
    exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
    echo "未找到 \`docker compose\`（Compose V2）。请先安装 Docker 与 compose 插件。" >&2
    exit 1
fi

echo -e "\033[1m安装目录（默认 /usr/share/microbin）：\033[0m"
read -r install_dir
install_dir=${install_dir:-/usr/share/microbin}

mkdir -p "$install_dir"
cd "$install_dir"

# .env 只在不存在时下载：重复执行脚本不应该把你改过的配置冲掉
if [ ! -f .env ]; then
    echo "下载 .env ..."
    fetch "${RAW}/.env" .env
else
    echo ".env 已存在，保留现有配置。"
fi

echo "下载 compose.yaml ..."
fetch "${RAW}/compose.yaml" compose.yaml

echo -e "\033[1m对外访问地址（例如 https://paste.example.com，留空表示不设置）：\033[0m"
read -r public_path

echo -e "\033[1m监听端口（默认 8080）：\033[0m"
read -r port
port=${port:-8080}

# 改 .env 里的一个键。
#
# 不能直接用 `sed -i "s|KEY=.*|KEY=值|"`：.env 里不少键是**注释掉**的
# （例如 `# export MICROBIN_PUBLIC_PATH=`），sed 只替换掉 `KEY=` 之后的部分，
# 行首的 `# ` 还在，结果仍然是被注释的一行 —— 用户填的域名静默失效。
# 这里连同行首的注释与 export 一起匹配，统一写成 `export KEY=值`；
# 键不存在时追加到文件末尾。
set_env() {
    local key="$1" val="$2" file="$3"
    awk -v k="$key" -v v="$val" '
        $0 ~ "^[[:space:]]*#?[[:space:]]*(export[[:space:]]+)?" k "=" {
            if (!done) { print "export " k "=" v; done = 1 }
            next
        }
        { print }
        END { if (!done) print "export " k "=" v }
    ' "$file" > "$file.tmp"
    mv "$file.tmp" "$file"
}

set_env MICROBIN_PORT "$port" .env

if [ -n "$public_path" ]; then
    # 去掉末尾斜杠：程序内部会自己拼路径，多余的斜杠会拼出 //upload
    public_path=${public_path%/}
    set_env MICROBIN_PUBLIC_PATH "$public_path" .env
fi

# 容器里的非 root 用户是 65532。用 named volume 时 Docker 会按镜像里的
# 属主初始化，不需要手工 chown；改用 bind mount 才需要（见 compose.yaml 注释）。
docker compose --env-file .env up --detach

echo
echo -e "\033[1m完成。\033[0m"
if [ -n "$public_path" ]; then
    echo "  访问地址：$public_path"
else
    echo "  访问地址：http://localhost:${port}（未设置 MICROBIN_PUBLIC_PATH，二维码与复制链接不可用）"
fi
echo "  配置文件：$install_dir/.env"
echo "  数据卷　：microbin_data（docker volume ls 可见）"

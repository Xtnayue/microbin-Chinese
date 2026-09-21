//! 国际化（i18n）支持。
//!
//! 设计要点：
//! * 语言在启动时确定（命令行参数 `--language` / 环境变量 `MICROBIN_LANGUAGE`），默认英文。
//! * 翻译表以 **英文原文作为键**，中文译文作为值。这样英文环境下是恒等映射，
//!   模板里直接书写英文原文即可，可读性最好；中文环境查表，查不到时回退英文。
//! * 模板中通过自定义 Askama filter 使用：`{{ "New"|t }}`。
//! * Rust 代码中通过 [`tr`] 使用。

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// 支持的语言。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    /// 解析语言代码，无法识别时回退到英文。
    pub fn from_code(code: &str) -> Lang {
        match code.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_cn" | "cn" | "zh-hans" | "zh-hant" | "chinese" => Lang::Zh,
            _ => Lang::En,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Zh => "zh",
        }
    }
}

/// 全局语言状态，首次访问时从命令行参数/环境变量解析。
static LANG: Lazy<Lang> = Lazy::new(|| Lang::from_code(&crate::args::ARGS.language));

/// 当前语言。
pub fn lang() -> Lang {
    *LANG
}

macro_rules! zh_table {
    ($($en:expr => $cn:expr),* $(,)?) => {{
        let mut m: HashMap<&'static str, &'static str> = HashMap::new();
        $( m.insert($en, $cn); )*
        m
    }};
}

/// 英文原文 -> 中文译文。
///
/// 注意：键必须与模板/代码中书写的英文原文完全一致（含标点与大小写）。
/// 仅存放需要翻译的条目；品牌名、编程语言名、环境变量名等技术标识不在此表中。
static ZH: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    zh_table! {
        // ---- 通用 / 导航 ----
        "New" => "新建",
        "List" => "列表",
        "Guide" => "指南",
        "Welcome to MicroBin" => "欢迎使用 MicroBin",
        "Sign in" => "登录",
        "Username" => "用户名",
        "Password" => "密码",
        "Go Home" => "返回首页",
        "Not Found" => "页面未找到",
        "Save" => "保存",
        "Update" => "更新",
        "Edit" => "编辑",
        "Remove" => "删除",
        "Copy" => "复制",
        "Copied" => "已复制",
        "Copy URL" => "复制链接",
        "Copy Text" => "复制文本",
        "Copy Redirect" => "复制跳转链接",
        "Download" => "下载",
        "Back to Upload" => "返回上传页",
        "QR" => "二维码",
        "Gallery" => "图库",
        "Stream" => "流式",
        "Filename" => "文件名",
        "Size" => "大小",
        "Action" => "操作",
        "Encrypted" => "已加密",
        "LOCKED" => "已锁定",
        "Created" => "创建时间",
        "Contents" => "内容",
        "Files" => "文件",
        "Redirect" => "跳转",
        "Content" => "内容",
        "Key" => "标识",
        "Valid" => "有效",
        "Hits" => "访问次数",
        "Status" => "状态",
        "Version" => "版本",
        "Info" => "信息",
        "Date" => "日期",
        "Description" => "描述",
        "Messages" => "消息",
        "Uploads" => "上传内容",
        "Links" => "链接",
        "Documentation and Help" => "文档与帮助",
        "Source Code" => "源代码",
        "Feedback" => "反馈",
        "Update available" => "有可用更新",
        "Update type" => "更新类型",
        "No updates available." => "暂无可用更新。",
        "URL Redirects" => "URL 跳转",
        "Environmental Variables" => "环境变量",
        "Argument" => "参数",
        "Value" => "值",
        "set" => "已启用",
        "unset" => "未启用",
        "Text" => "文本",
        "Image" => "图片",
        "Video" => "视频",
        "File" => "文件",
        "Priv." => "隐私",
        "Encryption" => "加密",
        "Edit." => "可编辑",
        "CLIENT" => "客户端",
        "SERVER" => "服务端",
        "Read" => "已阅读",
        "time, last" => "次，最近一次",
        "times, last" => "次，最近一次",
        "hits" => "次访问",
        "last" => "最近",

        // ---- 登录 / 鉴权 ----
        "Administrator username" => "管理员用户名",
        "Administrator password." => "管理员密码。",
        "Incorrect username or password." => "用户名或密码错误。",
        "Success!" => "成功！",
        "Okay" => "确定",
        "Incorrect password." => "密码错误。",
        "Incorrect password!" => "密码错误！",
        "Please enter the password to access or modify this upload." => "请输入密码以访问或修改此上传内容。",
        "Please enter your key to decrypt this upload." => "请输入密钥以解密此上传内容。",
        "Re-enter Password" => "再次输入密码",
        "Editing upload" => "正在编辑上传",

        // ---- 上传表单 ----
        "Options" => "选项",
        "Expiration" => "有效期",
        "Expire after" => "过期时间",
        "Burn after" => "阅后即焚",
        "Client-Rendered" => "浏览器端渲染",
        "Server-Rendered" => "服务端渲染",
        "Unencrypted (no password)" => "未加密（无密码）",
        "Unencrypted (protected)" => "未加密（受保护）",
        "Privacy" => "隐私",
        "Burn After" => "阅后即焚",
        "Syntax Highlighting" => "语法高亮",
        "Syntax" => "语法",
        "None" => "无",
        "Automatic" => "自动",
        "Public" => "公开",
        "Unlisted" => "不公开列出",
        "Read-only" => "只读",
        "Private" => "私有",
        "Secret" => "机密",
        "Select or drop file attachment" => "选择或拖放文件附件",
        "Type something here." => "在此输入内容。",
        "Uploader Password" => "上传密码",
        "Never Expire" => "永不过期",
        "No Limit" => "不限次数",
        "First Read" => "首次阅读后",
        "10th Read" => "第 10 次阅读后",
        "100th Read" => "第 100 次阅读后",
        "1000th Read" => "第 1000 次阅读后",
        "10000th Read" => "第 10000 次阅读后",
        "1 minute" => "1 分钟",
        "10 minutes" => "10 分钟",
        "1 hour" => "1 小时",
        "24 hours" => "24 小时",
        "3 days" => "3 天",
        "1 week" => "1 周",
        "1 month" => "1 个月",
        "6 months" => "6 个月",
        "1 year" => "1 年",
        "2 years" => "2 年",
        "4 years" => "4 年",
        "8 years" => "8 年",
        "16 years" => "16 年",

        // ---- 上传结果页 ----
        "Raw Text Content" => "原始文本内容",
        "Download all (Decrypted Zip)" => "下载全部（解密后的 Zip）",
        "Download all as ZIP" => "全部下载为 ZIP",
        "Decrypt text" => "解密文本",

        // ---- 列表页 ----
        "No uploads yet. 😔 Create one" => "还没有任何上传内容。😔 创建一个",
        "here" => "点这里",

        // ---- 前端 JS 动态文案 ----
        "Please select a file smaller than" => "文件过大，请选择小于",
        "Attached:" => "已附加：",
        "files" => "个文件",
        "Drop your file here" => "把文件拖到这里",
        "Drop your file here to replace" => "把文件拖到这里以替换",
        "Processing..." => "处理中…",
        "Failed to generate zip. See console." => "生成 zip 失败，请查看控制台。",

        // ---- 服务端返回给浏览器的提示 ----
        "Expiration exceeds maximum allowed" => "有效期超出允许的最大值",
        "Invalid UTF-8 in content" => "内容不是合法的 UTF-8 编码",
        "File exceeded size limit." => "文件超出大小限制。",
        "Failed to create attachment directory" => "创建附件目录失败",
        "Invalid login details." => "登录信息无效。",

        // ---- 页脚 ----
        "by Dániel Szabó and the FOSS Community. Let's keep the Web" => "由 Dániel Szabó 与 FOSS 社区打造。让我们一起保持网络的",
        "compact" => "紧凑",
        "accessible" => "易用",
        "and" => "与",
        "humane" => "人性化",

        // ---- 指南页 ----
        "Use the expiration dropdown to choose how long you want your upload to exist. When the selected time has expired, it will be removed from the server." => "使用「有效期」下拉框选择上传内容的保存时长。所选时间到期后，内容将从服务器上删除。",
        "Use the burn after dropdown to set a limit on how many times your data can be accessed before it will be removed from the server." => "使用「阅后即焚」下拉框设置数据可被访问的次数上限，达到上限后将从服务器上删除。",
        "Use the syntax highlighting dropdown to enable syntax highlighting for your upload, making it easier to read. You may choose to have the syntax highlighting done by your browser, which will also recognise the language automatically. You can select server-side highlighting, where you need to select the language yourself, but the code will get highlighting without javascript." => "使用「语法高亮」下拉框为上传内容启用语法高亮，便于阅读。你可以选择由浏览器完成高亮，浏览器会自动识别语言；也可以选择服务端高亮，此时需自行指定语言，但无需 JavaScript 即可获得高亮效果。",
        "Use the password field to set a password for your upload. This will encrypt your data while stored on our server with your password, and you will need to enter the password to access (in case of private and secret uploads) or to modify (in case of read-only uploads). Your password is encrypted, and in case of secret uploads, we never even see it." => "使用密码字段为上传内容设置密码。数据在服务器上会以你的密码加密存储；访问（私有与机密上传）或修改（只读上传）时都需要输入该密码。你的密码本身也是加密的，对于机密上传，我们甚至无法看到它。",
        "Use this dropdown to select the level of protection your upload needs. Use lower privacy levels if you or your organisation host MicroBin, and higher privacy levels if you are using a public MicroBin service." => "使用该下拉框选择上传内容所需的保护级别。如果 MicroBin 由你或你的组织自行托管，可使用较低的隐私级别；如果使用公共 MicroBin 服务，请选择较高的隐私级别。",
        "Level 1: Public" => "级别 1：公开",
        "This privacy level allows everyone to find, see, modify and remove your upload." => "该隐私级别允许任何人查找、查看、修改和删除你的上传内容。",
        "Level 2: Unlisted (recommended)" => "级别 2：不公开列出（推荐）",
        "Unlisted uploads cannot be found unless someone knows its unique, random identifier. If someone knows this identifier, they can see, modify and remove the upload." => "除非知道唯一的随机标识符，否则无法找到不公开列出的上传内容。若他人获得该标识符，则可以查看、修改和删除该上传内容。",
        "Level 3: Read-only" => "级别 3：只读",
        "With this privacy setting, the upload cannot be found unless someone knows its unique, random identifier. If someone knows this identifier, they can see the contents but cannot modify or remove it without entering the password of the upload." => "在此隐私设置下，除非知道唯一的随机标识符，否则无法找到该上传内容。若他人获得该标识符，可以查看内容，但必须输入上传密码才能修改或删除。",
        "Level 4: Private" => "级别 4：私有",
        "With this privacy setting, the upload cannot be found unless someone knows its unique, random identifier. If someone knows this identifier, they cannot see, modify or remove it without entering the password of the upload. Your upload and its attachments are encrypted, so they are stored safely." => "在此隐私设置下，除非知道唯一的随机标识符，否则无法找到该上传内容。若他人获得该标识符，也必须输入上传密码才能查看、修改或删除。你的上传内容及其附件均已加密，可安全存储。",
        "Level 5: Secret" => "级别 5：机密",
        "With this privacy setting, the upload cannot be found unless someone knows its unique, random identifier. If someone knows this identifier, they cannot see, modify or remove it without entering the password of the upload. Your browser sends us an already encrypted version, so the unencrypted data and password never even leave your device. This option requires you to enter your password many times when accessing your data, but is extremely safe." => "在此隐私设置下，除非知道唯一的随机标识符，否则无法找到该上传内容。若他人获得该标识符，也必须输入上传密码才能查看、修改或删除。你的浏览器只发送已加密的内容，因此明文数据和密码根本不会离开你的设备。该选项在访问数据时需要多次输入密码，但安全性极高。",
    }
});

/// 翻译英文原文。英文环境原样返回；中文环境查表，未命中时回退英文原文。
pub fn tr(en: &str) -> String {
    match lang() {
        Lang::En => en.to_string(),
        Lang::Zh => ZH.get(en).copied().unwrap_or(en).to_string(),
    }
}

// ---------------------------------------------------------------------------
// 命令行帮助（--help）的本地化
// ---------------------------------------------------------------------------

/// 在正式解析参数之前预判语言。
///
/// 原因：`--help` 由 clap 在解析参数的过程中输出，此时 [`crate::args::ARGS`] 尚未构造完成，
/// 因此不能依赖 ARGS。这里直接读环境变量并手动扫描 argv 中的 `--language`。
pub fn detect_lang_from_env() -> Lang {
    if let Ok(v) = std::env::var("MICROBIN_LANGUAGE") {
        return Lang::from_code(&v);
    }
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--language" {
            if let Some(v) = it.next() {
                return Lang::from_code(&v);
            }
        } else if let Some(v) = a.strip_prefix("--language=") {
            return Lang::from_code(v);
        }
    }
    Lang::En
}

/// 命令行参数的帮助文案：键为 clap 的参数 id（即结构体字段名），值为中文说明。
static ZH_HELP: &[(&str, &str)] = &[
    ("auth-basic-username", "启用 HTTP Basic 认证时要求的用户名"),
    ("auth-basic-password", "启用 HTTP Basic 认证时要求的密码"),
    ("auth-admin-username", "管理后台用户名"),
    ("auth-admin-password", "管理后台密码"),
    ("editable", "是否允许编辑上传内容"),
    ("footer-text", "自定义页脚文字"),
    ("hide-footer", "隐藏页脚"),
    ("hide-header", "隐藏顶部导航栏"),
    ("hide-logo", "隐藏 Logo"),
    ("no-listing", "禁用公开的列表页"),
    ("highlightsyntax", "启用语法高亮"),
    ("port", "监听端口"),
    ("bind", "监听地址"),
    ("private", "禁止创建不公开列出的上传"),
    ("pure-html", "使用纯 HTML 输出（不依赖自定义 CSS）"),
    ("json-db", "使用 JSON 文件作为数据库（默认使用 SQLite）"),
    ("public-path", "对外公开的完整路径前缀"),
    ("short-path", "短链接使用的路径前缀"),
    ("uploader-password", "上传者密码，设置后上传需要验证"),
    ("readonly", "上传后不可编辑"),
    ("show-read-stats", "显示阅读次数统计"),
    ("title", "页面标题"),
    ("threads", "工作线程数"),
    ("gc-days", "垃圾回收保留天数"),
    ("enable-burn-after", "启用「阅后即焚」功能"),
    ("default-burn-after", "默认的阅后即焚次数"),
    ("wide", "使用更宽的页面布局"),
    ("qr", "为上传内容显示二维码"),
    ("eternal-pasta", "允许创建永不过期的上传"),
    ("enable-readonly", "启用只读上传选项"),
    ("default-expiry", "默认有效期"),
    ("max-expiry", "允许的最大有效期"),
    ("data-dir", "数据存放目录"),
    ("no-file-upload", "禁用文件上传"),
    ("custom-css", "自定义 CSS 的地址"),
    ("hash-ids", "使用哈希作为上传标识"),
    ("list-server", "在公开列表中展示本服务器"),
    ("disable-telemetry", "关闭遥测"),
    ("disable-update-checking", "关闭更新检查"),
    ("encryption-client-side", "启用客户端加密"),
    ("encryption-server-side", "启用服务端加密"),
    ("default-privacy", "默认隐私级别"),
    ("max-file-size-encrypted-mb", "加密上传的最大文件大小（MB）"),
    ("max-file-size-unencrypted-mb", "未加密上传的最大文件大小（MB）"),
    ("default-view", "默认视图"),
    ("language", "界面语言：en（英文）或 zh（中文），默认 en"),
];

/// 按当前语言本地化 clap 的命令定义。
///
/// 英文环境下原样返回，保持与上游一致；中文环境下为每个参数补充中文说明。
pub fn localize_command(cmd: clap::Command<'static>) -> clap::Command<'static> {
    if detect_lang_from_env() == Lang::En {
        return cmd;
    }

    // clap 的 Id 无法读回字符串，这里用 long 名称反推 id 是否存在，避免 mut_arg 误加新参数。
    let longs: std::collections::HashSet<String> = cmd
        .get_arguments()
        .filter_map(|a| a.get_long().map(|s| s.to_string()))
        .collect();

    let mut cmd = cmd;
    for (id, help) in ZH_HELP.iter() {
        if longs.contains(*id) {
            cmd = cmd.mut_arg(*id, move |a| a.help(*help));
        }
    }
    cmd
}

/// 供 Askama 模板调用的自定义 filter。
///
/// 模板中的用法：`{{ "New"|t }}`
pub mod filters {
    use std::fmt::Display;

    /// 把英文原文翻译为当前语言。
    pub fn t<S: Display>(value: S) -> askama::Result<String> {
        Ok(crate::i18n::tr(&value.to_string()))
    }
}

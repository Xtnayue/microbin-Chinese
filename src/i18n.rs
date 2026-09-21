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
        "Editing upload" => "正在编辑上传",
        "Read password" => "read 密码",
        "Write password" => "write 密码",
        "Read permission requires a password." => "read 权限必须设置密码。",
        "Write permission requires a password." => "write 权限必须设置密码。",
        "Please enter the read password to access this upload." => "请输入 read 密码以访问此上传内容。",
        "Please enter the write password to modify or remove this upload." => "请输入 write 密码以修改或删除此上传内容。",
        "Anyone (no password)" => "任何人（无需密码）",
        "Requires a password" => "需要密码",
        "Refused (cannot be modified or removed)" => "拒绝写入（不可修改或删除）",
        "Listed publicly" => "公开列出",
        "Not listed" => "不公开列出",

        // ---- 上传表单 ----
        "Options" => "选项",
        "Expiration" => "有效期",
        "Expire after" => "过期时间",
        "Burn after" => "阅后即焚",
        "Client-Rendered" => "浏览器端渲染",
        "Server-Rendered" => "服务端渲染",
        "Burn After" => "阅后即焚",
        "Syntax Highlighting" => "语法高亮",
        "Syntax" => "语法",
        "None" => "无",
        "Automatic" => "自动",
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
        "Download all as ZIP" => "全部下载为 ZIP",

        // ---- 列表页 ----
        "No uploads yet. 😔 Create one" => "还没有任何上传内容。😔 创建一个",
        "here" => "点这里",

        // ---- 前端 JS 动态文案 ----
        "Please select a file smaller than" => "文件过大，请选择小于",
        "Attached:" => "已附加：",
        "files" => "个文件",
        "Drop your file here" => "把文件拖到这里",
        "Drop your file here to replace" => "把文件拖到这里以替换",

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
        "Permissions" => "权限",
        "Every upload carries three independent permissions: read, write and list. Unlike the older fixed privacy levels, each one is granted on its own, so you can combine them freely. Each permission answers exactly one question: read — who can see the content; write — who can modify or remove it; list — whether it shows up in the public list." => "每次上传都带有三项相互独立的权限：read、write 与 list。与早期固定搭配的「隐私级别」不同，每一项都单独授予，可以自由组合。每项权限只回答一个问题：read —— 谁能看到内容；write —— 谁能修改或删除；list —— 它会不会出现在公开列表中。",
        "Read permission" => "read 权限",
        "anyone who knows the identifier can view the content. No password is needed." => "任何知道标识符的人都能查看内容，无需密码。",
        "a read password is required. The text content and all attachments are stored encrypted on the server with that password, so nobody can read them without it, not even from a copy of the database." => "必须提供 read 密码。正文与全部附件都以该密码加密存储在服务器上，没有密码谁也读不出来，即使拿到数据库副本也一样。",
        "Scope: the upload page (/upload/...), the raw text view (/raw/...), single file downloads (/file/...), the zip download (/archive/...) and the editor — you must be able to read the content before you can edit it." => "作用范围：上传页（/upload/…）、原始文本（/raw/…）、单个文件下载（/file/…）、打包下载（/archive/…）以及编辑页 —— 编辑之前必须先能看到内容。",
        "Write permission" => "write 权限",
        "anyone who knows the identifier can edit the content and remove the upload." => "任何知道标识符的人都能修改内容、删除该上传。",
        "a write password is required to edit or remove the upload. This password only decides whether you are allowed to write; it is never used to encrypt anything." => "修改或删除该上传时必须提供 write 密码。该密码只用于判断「是否允许写入」，不参与任何加密。",
        "writing is refused altogether. Nobody can edit or remove the upload. The server administrator can still remove it with the administrator password." => "一律拒绝写入，任何人都无法修改或删除该上传；服务器管理员仍可用管理员密码删除。",
        "Scope: saving an edit (/edit/...), removing an upload (/remove/...), and whether the edit and remove links are shown at all. Note that editing needs both permissions: read to see the current content and write to save it." => "作用范围：保存编辑（/edit/…）、删除上传（/remove/…），以及页面上是否显示「编辑」「删除」入口。注意编辑同时需要两项权限：read 用来看到当前内容，write 用来保存。",
        "List permission" => "list 权限",
        "the upload is listed on the public /list page." => "该上传会出现在公开的 /list 列表页上。",
        "the upload is not listed, but it is still reachable by anyone who knows its identifier, subject to the read permission." => "该上传不出现在列表中，但知道标识符的人依然可以访问，能否看到内容取决于 read 权限。",
        "Scope: only the public /list page. The edit and remove links on that page are still governed by the write permission." => "作用范围：仅影响公开的 /list 列表页。列表页上的「编辑」「删除」入口仍然由 write 权限决定。",
        "How passwords are verified" => "密码的校验方式",
        "Passwords are never stored. When an upload is created, the server derives a verification value from the upload identifier and the password you typed, and stores only that value. To check a password later it decrypts the stored value with the submitted password: the check passes when the result is exactly the identifier of that upload." => "密码本身从不落库。创建上传时，服务器由「上传的标识符 + 你输入的密码」算出一个校验值，只保存这个校验值。之后校验密码时，用提交上来的密码去解密该校验值：结果恰好等于这个上传的标识符就算通过。",
        "This has two advantages: it works even when the text content is empty, which happens whenever an upload only carries attachments; and the cost of checking does not depend on how large the content is." => "这样做有两个好处：正文为空的上传（只带附件）也能正常校验；校验的开销与内容大小无关。",
        "The read password has a second job: when read is set to password, the text content and the attachments are encrypted with it, so the same password both passes the check and unlocks the content. The write password has no such second job — it is purely an authorisation check and never encrypts anything. The two passwords are independent and may be the same or different." => "read 密码还有第二个用途：当 read 设为 password 时，正文与附件就是用这个密码加密的，因此同一个密码既能通过校验、也能解开内容。write 密码没有第二个用途 —— 它纯粹是授权判断，不参与加密。两个密码彼此独立，可以相同，也可以不同。",
        "The server keeps neither the plaintext password nor the plaintext content. If you forget a read password, the content it protects cannot be recovered." => "服务器既不保存明文密码，也不保存明文内容。如果忘记 read 密码，受它保护的内容将无法恢复。",
        "Recommended combinations" => "推荐组合",
        "Because the three permissions are independent, the same effect as an old privacy level is just one particular combination. A few common ones:" => "三项权限相互独立，因此过去某个隐私级别的效果，只是这些权限的一种特定组合。以下是几种常见组合：",
        "What you want" => "想要的效果",
        "A fully public pasteboard" => "完全公开的粘贴板",
        "Share by link, but keep it out of the list" => "只靠链接分享，不出现在列表里",
        "Read-only share: anyone with the link may look, nobody may change" => "只读分享：拿到链接的人可以看，谁都不能改",
        "Anyone may read, only the password holder may change" => "谁都能读，只有持密码者可改",
        "Private: both reading and writing need a password" => "私密：读与写都需要密码",
        "Private and locked: readable with a password, never writable" => "私密且锁定：凭密码可读，永不可写",
        "Listed but still private: the list shows the link, the content still needs the password" => "私密但公开列出：列表里能看到链接，内容仍需密码",
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
    ("enable-unlisted", "是否允许把上传的 list 权限设为 no（不公开列出）"),
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
    ("enable-read-password", "是否允许把上传的 read 权限设为 password"),
    ("enable-write-password", "是否允许把上传的 write 权限设为 password"),
    ("default-expiry", "默认有效期"),
    ("max-expiry", "允许的最大有效期"),
    ("data-dir", "数据存放目录"),
    ("no-file-upload", "禁用文件上传"),
    ("custom-css", "自定义 CSS 的地址"),
    ("hash-ids", "使用哈希作为上传标识"),
    ("list-server", "在公开列表中展示本服务器"),
    ("disable-telemetry", "关闭遥测"),
    ("disable-update-checking", "关闭更新检查"),
    ("default-permissions", "新建上传时的默认权限组合，如 read=allow,write=allow,list=yes"),
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

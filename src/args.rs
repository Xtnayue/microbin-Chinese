use clap::{CommandFactory, FromArgMatches, Parser};
use lazy_static::lazy_static;
use serde::Serialize;
use std::convert::Infallible;
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;
use crate::fs;
use crate::pasta::{ReadAccess, WriteAccess};

lazy_static! {
    pub static ref ARGS: Args = Args::parse_localized();
}

impl Args {
    /// 解析命令行参数。
    ///
    /// 与 `Args::parse()` 的区别：先用 [`crate::i18n::localize_command`] 按语言
    /// 本地化 `--help` 文案，再交给 clap 解析。
    pub fn parse_localized() -> Args {
        let cmd = crate::i18n::localize_command(
            <Args as CommandFactory>::command::<'static>(),
        );
        let matches = cmd.get_matches();
        <Args as FromArgMatches>::from_arg_matches(&matches)
            .expect("failed to parse command line arguments")
    }
}

#[derive(Parser, Debug, Clone, Serialize)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    // 五个 `SecretArg` 都加了 `hide_env_values`：clap 默认会把环境变量的**值**
    // 原样打进 `--help`（`[env: MICROBIN_ADMIN_PASSWORD=真实密码]`），
    // 任何把 `--help` 输出贴进工单或落盘的场景都会泄露密码。
    #[clap(long, env = "MICROBIN_BASIC_AUTH_USERNAME", hide_env_values = true)]
    pub auth_basic_username: Option<SecretArg>,

    #[clap(long, env = "MICROBIN_BASIC_AUTH_PASSWORD", hide_env_values = true)]
    pub auth_basic_password: Option<SecretArg>,

    #[clap(long, env = "MICROBIN_ADMIN_USERNAME", default_value = "admin", hide_env_values = true)]
    pub auth_admin_username: SecretArg,

    #[clap(
        long,
        env = "MICROBIN_ADMIN_PASSWORD",
        default_value = "m1cr0b1n",
        hide_env_values = true,
        hide_default_value = true
    )]
    pub auth_admin_password: SecretArg,

    #[clap(long, env = "MICROBIN_EDITABLE", action = clap::ArgAction::Set, default_value_t = true)]
    pub editable: bool,

    #[clap(long, env = "MICROBIN_FOOTER_TEXT")]
    pub footer_text: Option<String>,

    #[clap(long, env = "MICROBIN_HIDE_FOOTER")]
    pub hide_footer: bool,

    #[clap(long, env = "MICROBIN_HIDE_HEADER")]
    pub hide_header: bool,

    #[clap(long, env = "MICROBIN_HIDE_LOGO")]
    pub hide_logo: bool,

    #[clap(long, env = "MICROBIN_NO_LISTING")]
    pub no_listing: bool,

    #[clap(long, env = "MICROBIN_HIGHLIGHTSYNTAX", action = clap::ArgAction::Set, default_value_t = true)]
    pub highlightsyntax: bool,

    #[clap(short, long, env = "MICROBIN_PORT", default_value_t = 8080)]
    pub port: u16,

    #[clap(short, long, env="MICROBIN_BIND", default_value_t = IpAddr::from([0, 0, 0, 0]))]
    pub bind: IpAddr,

    /// Whether an upload may be set to `list=no` (hidden from the public listing). This is also enforced server-side: posting `list=no` to /upload while the flag is false is rejected with HTTP 400.
    #[clap(long, env = "MICROBIN_ENABLE_UNLISTED", action = clap::ArgAction::Set, default_value_t = true)]
    pub enable_unlisted: bool,

    #[clap(long, env = "MICROBIN_PURE_HTML")]
    pub pure_html: bool,

    #[clap(long, env = "MICROBIN_JSON_DB")]
    pub json_db: bool,

    #[clap(long, env = "MICROBIN_PUBLIC_PATH")]
    pub public_path: Option<PublicUrl>,

    #[clap(long, env = "MICROBIN_SHORT_PATH")]
    pub short_path: Option<PublicUrl>,

    #[clap(long, env = "MICROBIN_UPLOADER_PASSWORD", hide_env_values = true)]
    pub uploader_password: Option<SecretArg>,

    #[clap(long, env = "MICROBIN_READONLY")]
    pub readonly: bool,

    #[clap(long, env = "MICROBIN_SHOW_READ_STATS", action = clap::ArgAction::Set, default_value_t = true)]
    pub show_read_stats: bool,

    #[clap(long, env = "MICROBIN_TITLE")]
    pub title: Option<String>,

    #[clap(short, long, env = "MICROBIN_THREADS", default_value_t = 1)]
    pub threads: u8,

    #[clap(short, long, env = "MICROBIN_GC_DAYS", default_value_t = 90)]
    pub gc_days: u16,

    #[clap(long, env = "MICROBIN_ENABLE_BURN_AFTER", action = clap::ArgAction::Set, default_value_t = true)]
    pub enable_burn_after: bool,

    #[clap(short, long, env = "MICROBIN_DEFAULT_BURN_AFTER", default_value_t = 0)]
    pub default_burn_after: u16,

    // `min_values(0)` + `default_missing_value` 让三种写法都成立：
    // 不传（默认 true）、裸 `--wide`（true）、`--wide=false`（false）。
    // 只写 `action = Set` 会让裸 `--wide` 报「缺少取值」。
    // `require_equals` 只影响取值写法：`--wide=false` 可以、`--wide false` 不行，
    // 好处是 `--help` 里显示成 `--wide[=<WIDE>]` 而不是误导性的 `[<WIDE>...]`。
    // （clap 3 没有 `num_args`，那是 clap 4 的 API。）
    //
    // 注意 doc comment 只能是一段（中间不能有空行），否则 clap 会把它当
    // long_help，`--help` 优先显示 long_help，中文覆盖就失效了。
    /// Fill the whole browser window. Set to false to constrain the page to a centred column, whose width comes from --max-width (1600px when that is 0).
    #[clap(
        long,
        env = "MICROBIN_WIDE",
        action = clap::ArgAction::Set,
        min_values = 0,
        max_values = 1,
        require_equals = true,
        default_missing_value = "true",
        default_value_t = true
    )]
    pub wide: bool,

    /// Maximum page width in pixels when not filling the window. 0 (the default) means no limit. A positive value always wins over --wide.
    #[clap(long, env = "MICROBIN_MAX_WIDTH", default_value_t = 0)]
    pub max_width: u32,

    #[clap(long, env = "MICROBIN_QR")]
    pub qr: bool,

    #[clap(long, env = "MICROBIN_ETERNAL_PASTA")]
    pub eternal_pasta: bool,

    /// Whether the `write` permission may be set to `password`. This is also enforced server-side: posting `write=password` to /upload while the flag is false is rejected with HTTP 400.
    #[clap(long, env = "MICROBIN_ENABLE_WRITE_PASSWORD", action = clap::ArgAction::Set, default_value_t = true)]
    pub enable_write_password: bool,

    /// Whether the `read` permission may be set to `password`. This is also enforced server-side: posting `read=password` to /upload while the flag is false is rejected with HTTP 400.
    #[clap(long, env = "MICROBIN_ENABLE_READ_PASSWORD", action = clap::ArgAction::Set, default_value_t = true)]
    pub enable_read_password: bool,

    #[clap(long, env = "MICROBIN_DEFAULT_EXPIRY", default_value = "24hour")]
    pub default_expiry: String,

    #[clap(long, env = "MICROBIN_MAX_EXPIRY", default_value = "1week")]
    pub max_expiry: String,

    #[clap(long, env = "MICROBIN_DATA_DIR", default_value = "microbin_data")]
    pub data_dir: String,

    #[clap(short, long, env = "MICROBIN_NO_FILE_UPLOAD")]
    pub no_file_upload: bool,

    #[clap(long, env = "MICROBIN_CUSTOM_CSS")]
    pub custom_css: Option<String>,

    #[clap(long, env = "MICROBIN_HASH_IDS")]
    pub hash_ids: bool,

    #[clap(long, env = "MICROBIN_LIST_SERVER")]
    pub list_server: bool,

    #[clap(long, env = "MICROBIN_DISABLE_TELEMETRY")]
    pub disable_telemetry: bool,

    #[clap(long, env = "MICROBIN_DISABLE_UPDATE_CHECKING")]
    pub disable_update_checking: bool,

    /// Default permissions for new uploads, comma-separated `key=value`, e.g. `read=allow,write=password,list=no`.
    #[clap(long, env = "MICROBIN_DEFAULT_PERMISSIONS", default_value = "read=allow,write=allow,list=yes")]
    pub default_permissions: String,

    #[clap(
        long,
        env = "MICROBIN_MAX_FILE_SIZE_ENCRYPTED_MB",
        default_value_t = 256
    )]
    pub max_file_size_encrypted_mb: usize,

    #[clap(
        long,
        env = "MICROBIN_MAX_FILE_SIZE_UNENCRYPTED_MB",
        default_value_t = 2048
    )]
    pub max_file_size_unencrypted_mb: usize,



    /// Default attachment layout on the upload page: gallery, list or stream. An explicit `?view=` in the URL still wins; unrecognised values fall back to gallery.
    #[clap(long, env = "MICROBIN_DEFAULT_VIEW", default_value = "gallery")]
    pub default_view: String,

    /// Fallback interface language: en (English) or zh (Chinese). This is only used when a visitor expresses no preference -- each request can override it via the microbin_lang cookie, an Accept-Language header, or a ?lang= query parameter.
    #[clap(long, env = "MICROBIN_LANGUAGE", default_value = "en")]
    pub language: String,
}

impl Args {
    /// 页面内容的最大宽度，直接作为 CSS `max-width` 的值。
    ///
    /// 优先级：`--max-width`（>0 时）> `--wide` > 兜底 1600px。
    /// `--max-width` 是显式给了数字，所以它压过 `--wide`；`--wide=false`
    /// 只是「别铺满」，具体多宽用默认的 1600px。
    ///
    /// 返回 `String` 而非 `&str`：`max_width` 是运行时数值，没法写成 `'static`。
    /// 注入是安全的 —— `u32` 格式化只会产生数字，后缀是常量。
    pub fn max_width_css(&self) -> String {
        if self.max_width > 0 {
            format!("{}px", self.max_width)
        } else if self.wide {
            "100%".to_string()
        } else {
            "1600px".to_string()
        }
    }

    pub fn public_path_as_str(&self) -> String {
        if self.public_path.is_some() {
            self.public_path.as_ref().unwrap().to_string()
        } else {
            String::from("")
        }
    }

    pub fn short_path_as_str(&self) -> String {
        if self.short_path.is_some() {
            self.short_path.as_ref().unwrap().to_string()
        } else if self.public_path.is_some() {
            self.public_path.as_ref().unwrap().to_string()
        } else {
            String::from("")
        }
    }

    /// 新建上传页附件视图的默认选项。
    ///
    /// `--default-view` 的取值会被**直接插进页面 JS 的字符串字面量**，因此这里做
    /// 白名单校验：只接受与视图选择器一致的三个取值，其余一律回退 `gallery`。
    /// 这样运维即使传进来一个带引号的值，也不会破坏整页脚本。
    pub fn default_view_as_str(&self) -> &'static str {
        match self.default_view.as_str() {
            "stream" => "stream",
            "list" => "list",
            _ => "gallery",
        }
    }

    /// 启动时自检：找出**自相矛盾**的配置组合，返回给调用方打印。
    ///
    /// 目前只覆盖一类问题：`--default-permissions` 里指定的默认档位，恰好是某个
    /// 被 `--enable-*` 关掉的选项。此时新建表单不会渲染那个选项，浏览器会回退到
    /// 同组的第一个选项，于是管理员配置的默认值被**静默改写**成另一个值。
    ///
    /// 这里只报告、不修改行为：默认值该不该生效是运维的决定，程序只负责让他知道。
    /// 返回值按固定顺序排列，便于测试断言。
    pub fn config_warnings(&self) -> Vec<&'static str> {
        let defaults = self.default_permissions();
        let mut warnings = Vec::new();

        if !self.enable_read_password && defaults.read == ReadAccess::Password {
            warnings.push(
                "MICROBIN_DEFAULT_PERMISSIONS asks for read=password, but the read \
                 password option is disabled (MICROBIN_ENABLE_READ_PASSWORD=false). \
                 The new-upload form will fall back to read=allow."
            );
        }

        if !self.enable_write_password && defaults.write == WriteAccess::Password {
            warnings.push(
                "MICROBIN_DEFAULT_PERMISSIONS asks for write=password, but the write \
                 password option is disabled (MICROBIN_ENABLE_WRITE_PASSWORD=false). \
                 The new-upload form will fall back to write=allow."
            );
        }

        if !self.enable_unlisted && !defaults.listed {
            warnings.push(
                "MICROBIN_DEFAULT_PERMISSIONS asks for list=no, but unlisted uploads \
                 are disabled (MICROBIN_ENABLE_UNLISTED=false). \
                 The new-upload form will fall back to list=yes."
            );
        }

        warnings
    }

    pub fn max_expiry_index(&self) -> usize {
        let options = &[
            "1min",
            "10min",
            "1hour",
            "24hour",
            "3days",
            "1week",
            "1month",
            "6months",
            "1year",
            "2years",
            "4years",
            "8years",
            "16years",
            "never",
        ];
        options.iter().position(|&x| x == self.max_expiry).unwrap_or(5)
    }

    /// 解析 `--default-permissions`，得到新建上传时的默认权限组合。
    ///
    /// 语法为逗号分隔的 `键=值`，键为 `read` / `write` / `list`（`listed` 亦可）。
    /// 未出现的键保留默认值；值无法识别时取该项最保守的一档。
    pub fn default_permissions(&self) -> DefaultPermissions {
        let mut parsed = DefaultPermissions::default();

        for part in self.default_permissions.split(',') {
            let Some((key, value)) = part.split_once('=') else {
                continue;
            };

            match key.trim().to_ascii_lowercase().as_str() {
                "read" => parsed.read = ReadAccess::from_form(value),
                "write" => parsed.write = WriteAccess::from_form(value),
                "list" | "listed" => {
                    parsed.listed = !matches!(
                        value.trim().to_ascii_lowercase().as_str(),
                        "no" | "false" | "off"
                    )
                }
                _ => {}
            }
        }

        parsed
    }

    pub fn without_secrets(self) -> Args {
        Args {
            auth_basic_username: None,
            auth_basic_password: None,
            auth_admin_username: crate::args::SecretArg(String::from("")),
            auth_admin_password: crate::args::SecretArg(String::from("")),
            editable: self.editable,
            footer_text: self.footer_text,
            hide_footer: self.hide_footer,
            hide_header: self.hide_header,
            hide_logo: self.hide_logo,
            no_listing: self.no_listing,
            highlightsyntax: self.highlightsyntax,
            port: self.port,
            bind: self.bind,
            enable_unlisted: self.enable_unlisted,
            pure_html: self.pure_html,
            json_db: self.json_db,
            public_path: self.public_path,
            short_path: self.short_path,
            uploader_password: None,
            readonly: self.readonly,
            show_read_stats: self.show_read_stats,
            title: self.title,
            list_server: self.list_server,
            threads: self.threads,
            gc_days: self.gc_days,
            enable_burn_after: self.enable_burn_after,
            default_burn_after: self.default_burn_after,
            wide: self.wide,
            max_width: self.max_width,
            qr: self.qr,
            eternal_pasta: self.eternal_pasta,
            enable_write_password: self.enable_write_password,
            enable_read_password: self.enable_read_password,
            default_expiry: self.default_expiry,
            max_expiry: self.max_expiry,
            data_dir: String::from(""),
            no_file_upload: self.no_file_upload,
            custom_css: self.custom_css,
            hash_ids: self.hash_ids,
            disable_telemetry: self.disable_telemetry,
            default_permissions: self.default_permissions,
            max_file_size_encrypted_mb: self.max_file_size_encrypted_mb,
            max_file_size_unencrypted_mb: self.max_file_size_unencrypted_mb,
            disable_update_checking: self.disable_update_checking,

            default_view: self.default_view,
            language: self.language,
        }
    }
}

/// `--default-permissions` 的解析结果：新建上传时预选的权限组合。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DefaultPermissions {
    pub read: ReadAccess,
    pub write: WriteAccess,
    pub listed: bool,
}

impl Default for DefaultPermissions {
    fn default() -> Self {
        Self {
            read: ReadAccess::Allow,
            write: WriteAccess::Allow,
            listed: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicUrl(pub String);
impl fmt::Display for PublicUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretArg(pub String);

impl FromStr for SecretArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(path) = s.strip_prefix("file://").or_else(|| s.strip_prefix("file:")) {
            // Read value from file
            fs::read_to_string(path)
                .map(|content| SecretArg(content.trim().to_string()))
                .map_err(|e| format!("Failed to read secret from {}: {}", path, e))
        } else {
            // Use the raw string provided
            Ok(SecretArg(s.to_string()))
        }
    }
}

// Allow it to be treated like a string automatically
impl std::ops::Deref for SecretArg {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for PublicUrl {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uri = s.strip_suffix('/').unwrap_or(s).to_owned();
        Ok(PublicUrl(uri))
    }
}

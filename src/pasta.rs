use bytesize::ByteSize;
use chrono::{Datelike, Local, TimeZone, Timelike};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::args::ARGS;
use crate::i18n::{self, tr};
use crate::util::animalnumbers::to_animal_names;
use crate::util::hashids::to_hashids;
use crate::util::syntaxhighlighter::html_highlight;

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Clone)]
pub struct PastaFile {
    pub name: String,
    pub size: ByteSize,
}

impl PastaFile {
    pub fn from_unsanitized(path: &str) -> Result<Self, &'static str> {
        let path = Path::new(path);
        let name = path.file_name().ok_or("Path did not contain a file name")?;
        let name = name.to_string_lossy().replace(' ', "_");
        Ok(Self {
            name,
            size: ByteSize::b(0),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url_encoded_name(&self) -> String {
        utf8_percent_encode(&self.name, NON_ALPHANUMERIC).to_string()
    }

    pub fn display_name(&self) -> &str {
        self.name.strip_suffix(".enc").unwrap_or(&self.name)
    }

    pub fn is_image(&self) -> bool {
        let lowercase_name = self.name.to_lowercase();
        let extensions = [
            ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".ico", ".svg", ".tiff", ".tif",
            ".jfif", ".pjpeg", ".pjp", ".avif", ".jxl", ".heif",
        ];
        extensions.iter().any(|&ext| lowercase_name.ends_with(ext))
    }

    pub fn is_video(&self) -> bool {
        let lowercase_name = self.name.to_lowercase();
        let extensions = [
            ".mp4", ".mov", ".wmv", ".webm", ".avi", ".flv", ".mkv", ".mts",
        ];
        extensions.iter().any(|&ext| lowercase_name.ends_with(ext))
    }

    pub fn embeddable(&self) -> bool {
        self.is_image() || self.is_video()
    }

    pub fn extension(&self) -> String {
        std::path::Path::new(&self.name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("FILE")
            .to_uppercase()
    }
}

/// `read` 权限：读取该上传需要满足的条件。
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReadAccess {
    /// 任何人只要知道标识符即可读取。
    Allow,
    /// 必须提供 read 密码；正文与附件均以该密码加密存储。
    Password,
}

/// `write` 权限：修改或删除该上传需要满足的条件。
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WriteAccess {
    /// 任何人只要知道标识符即可修改/删除。
    Allow,
    /// 必须提供 write 密码。该密码只用于授权判定，不参与内容加密。
    Password,
    /// 一律拒绝写入。任何人都无法修改或删除（管理员密码除外）。
    Refuse,
}

impl ReadAccess {
    pub fn as_str(self) -> &'static str {
        match self {
            ReadAccess::Allow => "allow",
            ReadAccess::Password => "password",
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            ReadAccess::Allow => 0,
            ReadAccess::Password => 1,
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => ReadAccess::Password,
            _ => ReadAccess::Allow,
        }
    }

    /// 从表单值解析；无法识别时取最保守的 `Password`。
    pub fn from_form(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "allow" | "open" => ReadAccess::Allow,
            _ => ReadAccess::Password,
        }
    }
}

impl Default for ReadAccess {
    /// 反序列化旧数据时取最保守的一侧。
    fn default() -> Self {
        ReadAccess::Password
    }
}

impl WriteAccess {
    pub fn as_str(self) -> &'static str {
        match self {
            WriteAccess::Allow => "allow",
            WriteAccess::Password => "password",
            WriteAccess::Refuse => "refuse",
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            WriteAccess::Allow => 0,
            WriteAccess::Password => 1,
            WriteAccess::Refuse => 2,
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => WriteAccess::Allow,
            1 => WriteAccess::Password,
            _ => WriteAccess::Refuse,
        }
    }

    /// 从表单值解析；无法识别时取最保守的 `Refuse`。
    pub fn from_form(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "allow" | "open" => WriteAccess::Allow,
            "password" => WriteAccess::Password,
            _ => WriteAccess::Refuse,
        }
    }
}

impl Default for WriteAccess {
    /// 反序列化旧数据时取最保守的一侧。
    fn default() -> Self {
        WriteAccess::Refuse
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pasta {
    pub id: u64,
    pub content: String,
    pub file: Option<PastaFile>,
    #[serde(default)]
    pub attachments: Option<Vec<PastaFile>>,
    pub extension: String,

    // ---- 权限项（三项相互独立，可单独授予）----
    /// `read` 权限。
    #[serde(default)]
    pub read_access: ReadAccess,
    /// `write` 权限。
    #[serde(default)]
    pub write_access: WriteAccess,
    /// `list` 权限：是否出现在 `/list` 公开列表中。
    #[serde(default)]
    pub listed: bool,

    /// read 密码的校验值：`encrypt(id, read_password)`。
    #[serde(default)]
    pub read_key: Option<String>,
    /// write 密码的校验值：`encrypt(id, write_password)`。
    #[serde(default)]
    pub write_key: Option<String>,

    pub editable: bool,
    pub created: i64,
    pub expiration: i64,
    pub last_read: i64,
    pub read_count: u64,
    pub burn_after_reads: u64,
    pub pasta_type: String,
}

impl Pasta {
    pub fn id_as_animals(&self) -> String {
        if ARGS.hash_ids {
            to_hashids(self.id)
        } else {
            to_animal_names(self.id)
        }
    }

    pub fn has_file(&self) -> bool {
        self.file.is_some()
    }

    pub fn total_size_as_string(&self) -> String {
        let mut total_size_bytes = self.content.as_bytes().len();
        if let Some(file) = &self.file {
            total_size_bytes += file.size.as_u64() as usize;
        }
        if let Some(attachments) = &self.attachments {
            for attachment in attachments {
                total_size_bytes += attachment.size.as_u64() as usize;
            }
        }

        if total_size_bytes < 1024 {
            format!("{} B", total_size_bytes)
        } else if total_size_bytes < 1024 * 1024 {
            format!("{} KB", total_size_bytes / 1024)
        } else if total_size_bytes < 1024 * 1024 * 1024 {
            format!("{} MB", total_size_bytes / (1024 * 1024))
        } else {
            format!("{} GB", total_size_bytes / (1024 * 1024 * 1024))
        }
    }

    pub fn file_embeddable(&self) -> bool {
        // 受 read 权限保护时附件是密文，无法直接内嵌预览
        return self.has_file()
            && self.file.as_ref().unwrap().embeddable()
            && !self.read_requires_password();
    }

    // ---- 权限判定辅助（供模板与端点复用）----

    /// read 权限是否要求密码。
    pub fn read_requires_password(&self) -> bool {
        self.read_access == ReadAccess::Password
    }

    /// write 权限是否要求密码。
    pub fn write_requires_password(&self) -> bool {
        self.write_access == WriteAccess::Password
    }

    /// write 权限是否一律拒绝。
    pub fn write_refused(&self) -> bool {
        self.write_access == WriteAccess::Refuse
    }

    /// 是否允许写入（不含密码校验）。
    pub fn write_allowed(&self) -> bool {
        self.write_access != WriteAccess::Refuse
    }

    /// 是否允许编辑：服务端总开关开启，且该上传的 write 权限不是 refuse。
    pub fn can_edit(&self) -> bool {
        self.editable && self.write_allowed()
    }

    /// 是否允许删除：write 权限不是 refuse 时，页面上才展示删除入口。
    pub fn can_remove(&self) -> bool {
        self.write_allowed()
    }

    pub fn created_as_string(&self) -> String {
        Local.timestamp_opt(self.created, 0).map(|date| {
            format!(
                "{:02}-{:02} {:02}:{:02}",
                date.month(),
                date.day(),
                date.hour(),
                date.minute(),
            )
        }).earliest().unwrap_or_else(|| {
            log::error!("Failed to process created date");
            String::from("Unknow")
        })
    }

    pub fn expiration_as_string(&self) -> String {
        if self.expiration == 0 {
            String::from("Never")
        } else {
            Local.timestamp_opt(self.expiration, 0).map(|date| {
                format!(
                    "{:02}-{:02} {:02}:{:02}",
                    date.month(),
                    date.day(),
                    date.hour(),
                    date.minute(),
                )
            }).earliest().unwrap_or_else(|| {
                log::error!("Failed to process expiration");
                String::from("Never")
            })
        }
    }

    /// 距上次读取的完整文案，例如「10 seconds ago」/「10 秒前」。
    pub fn last_read_time_ago_as_string(&self) -> String {
        Self::ago_as_string(self.seconds_since_last_read(), false)
    }

    /// 同上，但单位用缩写（管理页表格列窄）。例如「10 s ago」/「10 秒前」。
    pub fn short_last_read_time_ago_as_string(&self) -> String {
        Self::ago_as_string(self.seconds_since_last_read(), true)
    }

    /// 距上次读取经过了多少秒。系统时钟早于 UNIX EPOCH 时按 0 处理。
    fn seconds_since_last_read(&self) -> i64 {
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;
        timenow - self.last_read
    }

    /// 相对时间的本地化。
    ///
    /// 英文是「N 单位 ago」，中文是「N 单位前」——后缀在单位词之后，位置随语言变，
    /// 所以只把**单位词**交给翻译表，后缀在这里按语言拼。整句没法塞进翻译表，
    /// 因为数字是变量。
    ///
    /// 上游把这两份逻辑各写了一遍（只有单位缩写不同），中文文案漏改就成了
    /// 「最近 45 s ago」。合并成一个实现，避免以后再漏一处。
    ///
    /// 阈值沿用上游的 `> 1`：正好 1 天会显示成「24 hours ago」，这是上游既有
    /// 行为，这里不改，免得中文与英文环境出现差异。
    fn ago_as_string(seconds_ago: i64, short: bool) -> String {
        let days = (seconds_ago / 86400) as u16;
        if days > 1 {
            return format_ago(days, if short { tr("d") } else { tr("days") });
        }

        let hours = (seconds_ago / 3600) as u16;
        if hours > 1 {
            return format_ago(hours, if short { tr("h") } else { tr("hours") });
        }

        let minutes = (seconds_ago / 60) as u16;
        if minutes > 1 {
            return format_ago(minutes, if short { tr("m") } else { tr("minutes") });
        }

        let seconds = seconds_ago as u16;
        if seconds > 1 {
            return format_ago(seconds, if short { tr("s") } else { tr("seconds") });
        }

        tr("just now")
    }

    pub fn last_read_days_ago(&self) -> u16 {
        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // get seconds since last read and convert it to days
        ((timenow - self.last_read) / 86400) as u16
    }

    pub fn content_syntax_highlighted(&self) -> String {
        html_highlight(&self.content, &self.extension)
    }

    pub fn content_not_highlighted(&self) -> String {
        html_highlight(&self.content, "txt")
    }

    pub fn content_escaped(&self) -> String {
        html_escape::encode_text(
            &self
                .content
                .replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace('$', "\\$"),
        )
        .to_string()
    }

    pub fn content_textarea_safe(&self) -> String {
        html_escape::encode_text(&self.content).to_string()
    }
}

impl fmt::Display for Pasta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// 「N 单位前」/「N 单位 ago」。`unit` 是**已经翻译过**的单位词。
///
/// 中英文的差别只在后缀位置，所以单位词由调用方查表，这里只负责拼装。
fn format_ago(n: u16, unit: String) -> String {
    match i18n::lang() {
        i18n::Lang::Zh => format!("{}{}前", n, unit),
        i18n::Lang::En => format!("{} {} ago", n, unit),
    }
}

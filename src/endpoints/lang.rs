//! 语言切换入口。
//!
//! `GET /lang/{code}?next=/list` —— 把选择写进 cookie 再跳回原页面。
//!
//! 之所以用「跳转 + cookie」而不是让每个页面自己带 `?lang=`：
//! 后者会让用户点第二个链接时语言又变回去（查询参数不跨页保留），
//! 而 cookie 能一直跟着用户。

use crate::i18n::{Lang, LANG_COOKIE};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{get, web, HttpRequest, HttpResponse};

#[get("/lang/{code}")]
pub async fn set_lang(req: HttpRequest, code: web::Path<String>) -> HttpResponse {
    // 认不出的语言码回退到运维设的兜底语言，而不是硬编码英文。
    let lang = Lang::try_from_code(&code).unwrap_or_else(crate::i18n::default_lang);

    HttpResponse::Found()
        .cookie(
            Cookie::build(LANG_COOKIE, lang.code())
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .finish(),
        )
        .append_header(("Location", safe_next(&req)))
        .finish()
}

/// 从 `?next=` 取出跳转目标，并做站内校验。
fn safe_next(req: &HttpRequest) -> String {
    for pair in req.query_string().split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if k != "next" {
            continue;
        }
        // 校验必须用**解码后**的值：`%2F%2Fevil.com` 只有解码后才会暴露成
        // 协议相对 URL。回写时再编码回 ASCII —— 解码结果可能含非 ASCII 字节
        // （`?next=/p/%E4%B8%AD`），裸着放进 Location 头不符合规范。
        let decoded = crate::i18n::percent_decode(v);
        return if is_local_path(&decoded) {
            to_header_value(&decoded)
        } else {
            "/".to_string()
        };
    }
    "/".to_string()
}

/// 只接受站内相对路径。
///
/// 拒绝 `//evil.com`（协议相对 URL）、`/\evil.com`（部分浏览器把反斜杠当 `/`）、
/// 以及任何控制字符（响应头注入）。解码只做一次，所以 `%252F%252F...` 这类
/// 二次编码会解成 `%2F%2F...`，不以 `/` 开头，同样被拒。
fn is_local_path(v: &str) -> bool {
    v.starts_with('/')
        && !v.starts_with("//")
        && !v.contains('\\')
        && !v.chars().any(|c| c.is_control())
}

/// 把校验过的路径重新编码成可安全放进响应头的 ASCII。
///
/// 只保留可打印 ASCII（并排除 `"` 与 `\`），其余字节一律百分号编码。
fn to_header_value(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    for b in v.bytes() {
        if (0x21..=0x7E).contains(&b) && b != b'"' && b != b'\\' {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

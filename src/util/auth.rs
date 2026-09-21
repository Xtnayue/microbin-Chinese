use actix_multipart::Multipart;
use actix_web::dev::ServiceRequest;
use actix_web::web::Bytes;
use actix_web::{error, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;
use futures::TryStreamExt;

use crate::args::ARGS;
use crate::pasta::{Pasta, ReadAccess, WriteAccess};
use crate::util::misc::{decrypt, encrypt};

pub async fn auth_validator(
    req: ServiceRequest,
    creds: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    match (
        ARGS.auth_basic_username.as_ref(),
        ARGS.auth_basic_password.as_ref(),
        creds.password(),
    ) {
        (Some(conf_user), Some(conf_pwd), Some(cred_pwd))
            if creds.user_id() == **conf_user && **conf_pwd == cred_pwd =>
        {
            Ok(req)
        }
        _ => Err((error::ErrorBadRequest(crate::i18n::tr("Invalid login details.")), req)),
    }
}

pub async fn password_from_multipart(mut payload: Multipart) -> Result<String, Error> {
    let mut password = String::new();

    while let Some(mut field) = payload.try_next().await? {
        if field.name() == Some("password") {
            let password_bytes = field.bytes(1024).await.unwrap_or(Ok(Bytes::new()))?;
            password = String::from_utf8_lossy(&password_bytes).to_string();
        }
    }
    Ok(password)
}

// ---------------------------------------------------------------------------
// 权限密码的生成与校验
// ---------------------------------------------------------------------------
//
// read / write 两个权限各自可以携带一个密码。密码本身不落库，落库的是校验值：
//
//     校验值 = encrypt(上传的 id, 密码)
//
// 校验时用提交上来的密码解密该校验值，结果等于该上传的 id 即视为通过。
// 这样做的两个好处：
//   1. 与正文是否为空无关 —— 纯附件上传（正文为空）也能正确校验；
//   2. 不需要真的去解密正文，校验开销与内容大小无关。

/// 由上传 id 与密码生成校验值。
pub fn permission_key(id: u64, password: &str) -> String {
    encrypt(&id.to_string(), password)
}

fn key_matches(stored: Option<&String>, id: u64, password: &str) -> bool {
    match stored {
        Some(key) if !key.is_empty() => {
            matches!(decrypt(key, password), Ok(value) if value == id.to_string())
        }
        _ => false,
    }
}

/// 校验 read 密码。仅当该上传的 read 权限为 `password` 时才可能通过。
pub fn verify_read_password(pasta: &Pasta, password: &str) -> bool {
    if pasta.read_access != ReadAccess::Password || password.is_empty() {
        return false;
    }

    if key_matches(pasta.read_key.as_ref(), pasta.id, password) {
        return true;
    }

    // 兼容旧数据：分档模型迁移过来的记录没有 read_key，
    // 此时退化为「用正文密文本身校验」。
    if pasta.read_key.as_ref().map_or(true, |k| k.is_empty()) && !pasta.content.is_empty() {
        return decrypt(&pasta.content, password).is_ok();
    }

    false
}

/// 校验 write 密码。仅当该上传的 write 权限为 `password` 时才可能通过。
pub fn verify_write_password(pasta: &Pasta, password: &str) -> bool {
    if pasta.write_access != WriteAccess::Password || password.is_empty() {
        return false;
    }

    key_matches(pasta.write_key.as_ref(), pasta.id, password)
}

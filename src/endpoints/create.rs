use crate::args::{Args, DefaultPermissions, ARGS};
use crate::pasta::{Pasta, PastaFile, ReadAccess, WriteAccess};
use crate::util::animalnumbers::to_animal_names;
use crate::util::auth;
use crate::util::db::insert;
use crate::util::hashids::to_hashids;
use crate::util::misc::{encrypt, encrypt_file, is_valid_url};
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::error::ErrorBadRequest;
use actix_web::cookie::Cookie;
use actix_web::{get, web, Error, HttpResponse, Responder};
use askama::Template;
use crate::i18n::filters;
use bytes::BytesMut;
use bytesize::ByteSize;
use futures::TryStreamExt;
use log::warn;
use rand::Rng;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    args: &'a Args,
    status: String,
    default_permissions: DefaultPermissions,
    max_expiry_index: usize,
}

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        IndexTemplate {
            args: &ARGS,
            status: String::from(""),
            default_permissions: ARGS.default_permissions(),
            max_expiry_index: ARGS.max_expiry_index(),
        }
        .render()
        .unwrap(),
    )
}

#[get("/{status}")]
pub async fn index_with_status(param: web::Path<String>) -> HttpResponse {
    let status = param.into_inner();

    return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        IndexTemplate {
            args: &ARGS,
            status,
            default_permissions: ARGS.default_permissions(),
            max_expiry_index: ARGS.max_expiry_index(),
        }
        .render()
        .unwrap(),
    );
}

const EXPIRATION_OPTIONS: &[&str] = &[
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

pub fn expiration_to_timestamp(expiration: &str, timenow: i64) -> i64 {
    match expiration {
        "1min" => timenow + 60,
        "10min" => timenow + 60 * 10,
        "1hour" => timenow + 60 * 60,
        "24hour" => timenow + 60 * 60 * 24,
        "3days" => timenow + 60 * 60 * 24 * 3,
        "1week" => timenow + 60 * 60 * 24 * 7,
        "1month" => timenow + 60 * 60 * 24 * 30,
        "6months" => timenow + 60 * 60 * 24 * 30 * 6,
        "1year" => timenow + 60 * 60 * 24 * 365,
        "2years" => timenow + 60 * 60 * 24 * 365 * 2,
        "4years" => timenow + 60 * 60 * 24 * 365 * 4,
        "8years" => timenow + 60 * 60 * 24 * 365 * 8,
        "16years" => timenow + 60 * 60 * 24 * 365 * 16,
        "never" => {
            if ARGS.eternal_pasta {
                0
            } else {
                timenow + 60 * 60 * 24 * 7
            }
        }
        _ => {
            log::error!("{}", "Unexpected expiration time!");
            timenow + 60 * 60 * 24 * 7
        }
    }
}

pub fn is_valid_expiration(expiration: &str, max_expiry: &str) -> bool {
    let max_index = EXPIRATION_OPTIONS.iter().position(|&x| x == max_expiry).unwrap_or(5);
    let current_index = EXPIRATION_OPTIONS.iter().position(|&x| x == expiration).unwrap_or(0);
    current_index <= max_index
}

pub async fn create(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => {
            log::error!("SystemTime before UNIX EPOCH!");
            0
        }
    } as i64;

    let mut new_pasta = Pasta {
        id: rand::thread_rng().gen::<u16>() as u64,
        content: String::from(""),
        file: None,
        extension: String::from(""),
        read_access: ReadAccess::Allow,
        write_access: WriteAccess::Allow,
        listed: true,
        read_key: None,
        write_key: None,
        editable: ARGS.editable,
        created: timenow,
        read_count: 0,
        burn_after_reads: 0,
        last_read: timenow,
        pasta_type: String::from(""),
        expiration: expiration_to_timestamp(&ARGS.default_expiry, timenow),
        attachments: None,
    };

    // 两个权限各自的密码。密码本身不落库，落库的是由 id 派生的校验值。
    let mut read_password = String::from("");
    let mut write_password = String::from("");
    let mut uploader_password = String::from("");

    while let Some(mut field) = payload.try_next().await? {
        let Some(field_name) = field.name() else {
            continue;
        };
        match field_name {
            "uploader_password" => {
                while let Some(chunk) = field.try_next().await? {
                    uploader_password
                        .push_str(std::str::from_utf8(&chunk).unwrap().to_string().as_str());
                }
                continue;
            }
            "read_access" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.read_access =
                        ReadAccess::from_form(std::str::from_utf8(&chunk).unwrap());
                }
                continue;
            }
            "read_password" => {
                while let Some(chunk) = field.try_next().await? {
                    read_password.push_str(std::str::from_utf8(&chunk).unwrap());
                }
                continue;
            }
            "write_access" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.write_access =
                        WriteAccess::from_form(std::str::from_utf8(&chunk).unwrap());
                }
                continue;
            }
            "write_password" => {
                while let Some(chunk) = field.try_next().await? {
                    write_password.push_str(std::str::from_utf8(&chunk).unwrap());
                }
                continue;
            }
            "listed" => {
                while let Some(chunk) = field.try_next().await? {
                    let value = std::str::from_utf8(&chunk).unwrap().trim().to_ascii_lowercase();
                    // 无法识别的取值一律按「不列出」处理
                    new_pasta.listed = matches!(value.as_str(), "yes" | "on" | "true" | "1");
                }
                continue;
            }
            "expiration" => {
                let mut expiration_str = String::new();
                while let Some(chunk) = field.try_next().await? {
                    expiration_str = std::str::from_utf8(&chunk).unwrap().to_string();
                }

                if !is_valid_expiration(&expiration_str, &ARGS.max_expiry) {
                    return Err(ErrorBadRequest(crate::i18n::tr("Expiration exceeds maximum allowed")));
                }

                new_pasta.expiration = expiration_to_timestamp(&expiration_str, timenow);
                continue;
            }
            "burn_after" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.burn_after_reads = match std::str::from_utf8(&chunk).unwrap() {
                        "1" => 1,
                        "10" => 10,
                        "100" => 100,
                        "1000" => 1000,
                        "10000" => 10000,
                        "0" => 0,
                        _ => {
                            log::error!("{}", "Unexpected burn after value!");
                            0
                        }
                    };
                }

                continue;
            }
            "content" => {
                let mut buf = BytesMut::new();
                while let Some(chunk) = field.try_next().await? {
                    buf.extend_from_slice(&chunk);
                }
                if !buf.is_empty() {
                    new_pasta.content = String::from_utf8(buf.to_vec())
                        .map_err(|_| ErrorBadRequest(crate::i18n::tr("Invalid UTF-8 in content")))?;
                    new_pasta.pasta_type = if is_valid_url(new_pasta.content.as_str()) {
                        String::from("url")
                    } else {
                        String::from("text")
                    };
                }
                continue;
            }
            "syntax_highlight" => {
                while let Some(chunk) = field.try_next().await? {
                    new_pasta.extension = std::str::from_utf8(&chunk).unwrap().to_string();
                }
                continue;
            }
            "file" => {
                if ARGS.no_file_upload {
                    continue;
                }

                let path = field.content_disposition().and_then(|cd| cd.get_filename());

                let path = match path {
                    Some("") => continue,
                    Some(p) => p,
                    None => continue,
                };

                let mut file = match PastaFile::from_unsanitized(path) {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("Unsafe file name: {e:?}");
                        continue;
                    }
                };

                if let Err(e) = std::fs::create_dir_all(format!(
                    "{}/attachments/{}",
                    ARGS.data_dir,
                    &new_pasta.id_as_animals()
                )) {
                    log::error!("Failed to create directory: {}", e);
                    return Err(actix_web::error::ErrorInternalServerError(
                        crate::i18n::tr("Failed to create attachment directory"),
                    ));
                }

                let filepath = format!(
                    "{}/attachments/{}/{}",
                    ARGS.data_dir,
                    &new_pasta.id_as_animals(),
                    &file.name()
                );

                let mut f = web::block(|| std::fs::File::create(filepath)).await??;
                let mut size = 0;
                while let Some(chunk) = field.try_next().await? {
                    size += chunk.len();
                    if (new_pasta.read_access == ReadAccess::Password
                        && size > ARGS.max_file_size_encrypted_mb * 1024 * 1024)
                        || size > ARGS.max_file_size_unencrypted_mb * 1024 * 1024
                    {
                        return Err(ErrorBadRequest(crate::i18n::tr("File exceeded size limit.")));
                    }
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                file.size = ByteSize::b(size as u64);

                if new_pasta.file.is_none() {
                    new_pasta.file = Some(file);
                } else {
                    if new_pasta.attachments.is_none() {
                        new_pasta.attachments = Some(Vec::new());
                    }
                    new_pasta.attachments.as_mut().unwrap().push(file);
                }
                
                new_pasta.pasta_type = String::from("text");
            }
            field => {
                log::error!("Unexpected multipart field:  {}", field);
            }
        }
    }

    if ARGS.readonly && ARGS.uploader_password.is_some() {
        if uploader_password.trim() != ARGS.uploader_password.as_ref().unwrap().trim() {
            log::warn!("Uploader password mismatch. Input length: {}, Expected length: {}", uploader_password.trim().len(), ARGS.uploader_password.as_ref().unwrap().trim().len());
            return Ok(HttpResponse::Found()
                .append_header(("Location", format!("{}/incorrect", ARGS.public_path_as_str())))
                .finish());
        }
    }

    let id = new_pasta.id;

    // ---- 应用三项权限 ----
    // read 权限：声明要密码却没给密码时，绝不静默降级为公开，直接拒绝。
    if new_pasta.read_access == ReadAccess::Password {
        if read_password.is_empty() {
            return Err(ErrorBadRequest(crate::i18n::tr(
                "Read permission requires a password.",
            )));
        }
        new_pasta.read_key = Some(auth::permission_key(id, &read_password));
    }

    // write 权限：只生成校验值，不参与任何内容加密。
    if new_pasta.write_access == WriteAccess::Password {
        if write_password.is_empty() {
            return Err(ErrorBadRequest(crate::i18n::tr(
                "Write permission requires a password.",
            )));
        }
        new_pasta.write_key = Some(auth::permission_key(id, &write_password));
    }

    // ---- 按 read 权限加密正文与附件 ----
    let read_protected = new_pasta.read_access == ReadAccess::Password;

    if read_protected && !new_pasta.content.is_empty() {
        new_pasta.content = encrypt(&new_pasta.content, &read_password);
    }

    if read_protected {
        let mut files_to_encrypt: Vec<&PastaFile> = Vec::new();
        if let Some(file) = &new_pasta.file {
            files_to_encrypt.push(file);
        }
        if let Some(attachments) = &new_pasta.attachments {
            for attachment in attachments {
                files_to_encrypt.push(attachment);
            }
        }

        for file in files_to_encrypt {
            let filepath = format!(
                "{}/attachments/{}/{}",
                ARGS.data_dir,
                &new_pasta.id_as_animals(),
                &file.name()
            );
            encrypt_file(&read_password, &filepath)
                .expect("Failed to encrypt file with the read password")
        }
    }

    {
        let mut pastas = data.pastas.lock().unwrap();
        pastas.push(new_pasta);

        for (_, pasta) in pastas.iter().enumerate() {
            if pasta.id == id {
                insert(Some(&pastas), Some(pasta));
            }
        }
    }

    let slug = if ARGS.hash_ids {
        to_hashids(id)
    } else {
        to_animal_names(id)
    };

    // read 权限要求密码时，直接引导用户去密码页；否则直接进详情页。
    if read_protected {
        Ok(HttpResponse::Found()
            .append_header(("Location", format!("{}/auth/{}/success", ARGS.public_path_as_str(), slug)))
            .finish())
    } else {
        // Generate time-limited token for initial view using Hashids
        let timenow = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiry = timenow + 15; // 15 seconds validity
        
        // Use global HARSH instance
        let encoded_token = crate::util::hashids::HARSH.encode(&[expiry, id]);

        Ok(HttpResponse::Found()
            .append_header((
                "Location",
                format!("{}/upload/{}", ARGS.public_path_as_str(), slug),
            ))
            .cookie(
                Cookie::build("owner_token", encoded_token)
                    .path("/")
                    .max_age(actix_web::cookie::time::Duration::seconds(15))
                    .finish(),
            )
            .finish())
    }
}

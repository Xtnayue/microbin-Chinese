use crate::args::ARGS;
use crate::pasta::Pasta;
use crate::util::animalnumbers::to_u64;
use crate::util::auth;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::{decrypt_file, remove_expired};
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use zip::write::SimpleFileOptions;

/// 在内存里查找上传的下标。
fn find_index(pastas: &[Pasta], id: &str) -> Option<usize> {
    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(id).unwrap_or(0)
    } else {
        to_u64(id).unwrap_or(0)
    };

    pastas.iter().position(|pasta| pasta.id == intern_id)
}

/// 把该上传的正文附件打包成 zip 返回。
///
/// * `password` 为 `None` 时，直接把磁盘上的文件塞进 zip —— 只有 read 权限为
///   `allow` 时才会走到这里，此时磁盘上的就是明文。
/// * `password` 为 `Some(..)` 时，文件在磁盘上是密文，逐个解密后以原始文件名
///   放进 zip。
fn build_zip(pasta: &Pasta, password: Option<&str>) -> Result<HttpResponse, Error> {
    let zip_file = tempfile::tempfile()?;
    let mut zip = zip::ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    let mut add_file_to_zip =
        |name: &str, path: PathBuf, password: Option<&str>| -> Result<(), std::io::Error> {
            let mut f = File::open(path)?;

            let contents = match password {
                Some(password) => decrypt_file(password, &f).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::Other, "Failed to decrypt file")
                })?,
                None => {
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer)?;
                    buffer
                }
            };

            zip.start_file(name, options)?;
            zip.write_all(&contents)?;
            Ok(())
        };

    let encrypted = password.is_some();

    // Add primary file
    if let Some(file) = &pasta.file {
        let final_path = if encrypted {
            let primary = PathBuf::from(format!(
                "{}/attachments/{}/{}.enc",
                ARGS.data_dir,
                pasta.id_as_animals(),
                file.name()
            ));
            if primary.exists() {
                primary
            } else {
                // 旧数据的兜底命名
                PathBuf::from(format!(
                    "{}/attachments/{}/data.enc",
                    ARGS.data_dir,
                    pasta.id_as_animals()
                ))
            }
        } else {
            PathBuf::from(format!(
                "{}/attachments/{}/{}",
                ARGS.data_dir,
                pasta.id_as_animals(),
                file.name()
            ))
        };

        if final_path.exists() {
            add_file_to_zip(file.name(), final_path, password)?;
        }
    }

    // Add attachments
    if let Some(attachments) = &pasta.attachments {
        for file in attachments {
            let final_path = PathBuf::from(format!(
                "{}/attachments/{}/{}{}",
                ARGS.data_dir,
                pasta.id_as_animals(),
                file.name(),
                if encrypted { ".enc" } else { "" }
            ));

            if final_path.exists() {
                add_file_to_zip(file.name(), final_path, password)?;
            }
        }
    }

    let mut zip_file = zip.finish().map_err(actix_web::error::ErrorInternalServerError)?;

    // Read the zip back into a buffer to send
    zip_file.seek(std::io::SeekFrom::Start(0))?;
    let mut buffer = Vec::new();
    zip_file.read_to_end(&mut buffer)?;

    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}.zip\"", pasta.id_as_animals()),
        ))
        .body(buffer))
}

#[get("/archive/{id}")]
pub async fn get_archive(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_index(&pastas[..], &id.into_inner()) {
        let pasta = &pastas[index];

        // read 权限要求密码时，先引导到密码页（提交后走 post_archive）
        if pasta.read_requires_password() {
            return Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!(
                        "{}/auth_file/{}?to=archive",
                        ARGS.public_path_as_str(),
                        pasta.id_as_animals()
                    ),
                ))
                .finish());
        }

        return build_zip(pasta, None);
    }

    Ok(HttpResponse::NotFound().finish())
}

/// 密码页提交后的落地处理：校验 read 密码，返回解密后的 zip。
#[post("/archive/{id}")]
pub async fn post_archive(
    id: web::Path<String>,
    data: web::Data<AppState>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let password = auth::password_from_multipart(payload).await?;

    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_index(&pastas[..], &id.into_inner()) {
        let pasta = &pastas[index];

        if pasta.read_requires_password() {
            if !auth::verify_read_password(pasta, &password) {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!(
                            "{}/auth_file/{}/incorrect",
                            ARGS.public_path_as_str(),
                            pasta.id_as_animals()
                        ),
                    ))
                    .finish());
            }

            return build_zip(pasta, Some(&password));
        }

        return build_zip(pasta, None);
    }

    Ok(HttpResponse::NotFound().finish())
}

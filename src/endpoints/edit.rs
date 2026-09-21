use crate::args::Args;
use crate::endpoints::errors::ErrorTemplate;
use crate::util::animalnumbers::to_u64;
use crate::util::auth;
use crate::util::db::update;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::{decrypt, encrypt, remove_expired};
use crate::{AppState, Pasta, ARGS};
use actix_multipart::Multipart;
use actix_web::error::ErrorBadRequest;
use actix_web::{get, post, web, Error, HttpResponse};
use askama::Template;
use crate::i18n::filters;
use bytes::BytesMut;
use futures::TryStreamExt;

#[derive(Template)]
#[template(path = "edit.html", escape = "none")]
struct EditTemplate<'a> {
    pasta: &'a Pasta,
    args: &'a Args,
    path: &'a String,
    status: &'a String,
}

/// 编辑表单提交上来的字段。
///
/// 编辑同时牵涉两项权限，因此表单里可能出现两个密码字段：
/// * `read_password` —— read 权限的密码。`read_access` 为 `password` 时，
///   正文以该密码加密存储，保存新内容时必须用同一个密码重新加密；
/// * `write_password` —— write 权限的密码。`write_access` 为 `password` 时，
///   必须通过它才能写入。
///
/// 另兼容旧的 `password` 字段名（等价于 `read_password`）。
struct EditForm {
    content: String,
    read_password: String,
    write_password: String,
}

impl EditForm {
    fn new() -> Self {
        Self {
            content: String::new(),
            read_password: String::new(),
            write_password: String::new(),
        }
    }
}

/// 解析编辑表单的 multipart 请求体。
async fn parse_edit_form(mut payload: Multipart) -> Result<EditForm, Error> {
    let mut form = EditForm::new();

    while let Some(mut field) = payload.try_next().await? {
        let name = match field.name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let mut buf = BytesMut::new();
        while let Some(chunk) = field.try_next().await? {
            buf.extend_from_slice(&chunk);
        }

        let value = String::from_utf8(buf.to_vec())
            .map_err(|_| ErrorBadRequest(crate::i18n::tr("Invalid UTF-8 in content")))?;

        match name.as_str() {
            "content" => form.content = value,
            "read_password" | "password" => form.read_password = value,
            "write_password" => form.write_password = value,
            _ => {}
        }
    }

    Ok(form)
}

/// 应用一次编辑，返回 `true` 表示已保存。
///
/// 两道校验：
/// 1. **write**：`refuse` 一律拒绝；`password` 必须提供正确的 write 密码。
/// 2. **read**：`password` 时必须提供正确的 read 密码，并用它重新加密正文
///    （正文与附件在存储时就是以 read 密码加密的）。
fn apply_edit(pastas: &mut Vec<Pasta>, index: usize, form: &EditForm) -> bool {
    if !pastas[index].editable || pastas[index].write_refused() {
        return false;
    }

    if pastas[index].write_requires_password()
        && !auth::verify_write_password(&pastas[index], &form.write_password)
    {
        return false;
    }

    let content = if pastas[index].read_requires_password() {
        if !auth::verify_read_password(&pastas[index], &form.read_password) {
            return false;
        }
        encrypt(&form.content, &form.read_password)
    } else {
        form.content.to_owned()
    };

    pastas[index].content.replace_range(.., &content);

    // save pasta in database
    update(Some(&*pastas), Some(&pastas[index]));

    true
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", location.to_string()))
        .finish()
}

fn error_page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

/// 进入编辑页前的统一判定：
/// * 服务端禁止编辑，或该上传的 write 权限为 `refuse` —— 无法编辑，回首页；
/// * read 权限要求密码 —— 先跳到密码页（拿到明文正文才谈得上编辑）；
/// * 否则直接渲染编辑表单。
fn edit_entry(pasta: &Pasta, status: &str) -> HttpResponse {
    if !pasta.editable || pasta.write_refused() {
        return redirect_to(&format!("{}/", ARGS.public_path_as_str()));
    }

    if pasta.read_requires_password() {
        let suffix = if status.is_empty() {
            String::new()
        } else {
            format!("/{}", status)
        };

        return redirect_to(&format!(
            "{}/auth_edit_private/{}{}",
            ARGS.public_path_as_str(),
            pasta.id_as_animals(),
            suffix
        ));
    }

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        EditTemplate {
            pasta,
            args: &ARGS,
            path: &String::from("edit"),
            status: &String::from(status),
        }
        .render()
        .unwrap(),
    )
}

#[get("/edit/{id}")]
pub async fn get_edit(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == id {
            return edit_entry(pasta, "");
        }
    }

    error_page()
}

#[get("/edit/{id}/{status}")]
pub async fn get_edit_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    remove_expired(&mut pastas);

    for pasta in pastas.iter() {
        if pasta.id == intern_id {
            return edit_entry(pasta, &status);
        }
    }

    error_page()
}

/// 密码页提交后的落地处理：校验 read 密码，然后把正文解密后交给编辑器。
///
/// 保存新内容时需要同一个 read 密码重新加密，因此编辑表单里还会再要一次
/// read 密码（模板中按 `pasta.read_requires_password()` 决定是否显示）。
#[post("/edit_private/{id}")]
pub async fn post_edit_private(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let password = auth::password_from_multipart(payload).await?;

    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    let index = match pastas.iter().position(|pasta| pasta.id == id) {
        Some(index) => index,
        None => return Ok(error_page()),
    };

    if !pastas[index].can_edit() {
        return Ok(redirect_to(&format!("{}/", ARGS.public_path_as_str())));
    }

    if pastas[index].read_requires_password()
        && !auth::verify_read_password(&pastas[index], &password)
    {
        return Ok(redirect_to(&format!(
            "{}/auth_edit_private/{}/incorrect",
            ARGS.public_path_as_str(),
            pastas[index].id_as_animals()
        )));
    }

    let original_content = pastas[index].content.to_owned();

    // 正文以 read 密码加密，这里临时解密以便在编辑器中显示明文
    if pastas[index].read_requires_password() && !original_content.is_empty() {
        if let Ok(plain) = decrypt(&original_content, &password) {
            pastas[index].content.replace_range(.., plain.as_str());
        }
    }

    // serve pasta in template
    let response = HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        EditTemplate {
            pasta: &pastas[index],
            args: &ARGS,
            path: &String::from("submit_edit_private"),
            status: &String::from(""),
        }
        .render()
        .unwrap(),
    );

    // 内存里恢复密文，避免明文被后续持久化
    pastas[index].content = original_content;

    Ok(response)
}

#[post("/submit_edit_private/{id}")]
pub async fn post_submit_edit_private(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let form = parse_edit_form(payload).await?;

    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    let index = match pastas.iter().position(|pasta| pasta.id == id) {
        Some(index) => index,
        None => return Ok(error_page()),
    };

    if !apply_edit(&mut pastas, index, &form) {
        return Ok(redirect_to(&format!(
            "{}/edit/{}/incorrect",
            ARGS.public_path_as_str(),
            pastas[index].id_as_animals()
        )));
    }

    Ok(redirect_to(&format!(
        "{}/auth/{}/success",
        ARGS.public_path_as_str(),
        pastas[index].id_as_animals()
    )))
}

#[post("/edit/{id}")]
pub async fn post_edit(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id.into_inner()).unwrap_or(0)
    };

    let form = parse_edit_form(payload).await?;

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let index = match pastas.iter().position(|pasta| pasta.id == id) {
        Some(index) => index,
        None => return Ok(error_page()),
    };

    if !apply_edit(&mut pastas, index, &form) {
        return Ok(redirect_to(&format!(
            "{}/edit/{}/incorrect",
            ARGS.public_path_as_str(),
            pastas[index].id_as_animals()
        )));
    }

    Ok(redirect_to(&format!(
        "{}/upload/{}",
        ARGS.public_path_as_str(),
        pastas[index].id_as_animals()
    )))
}

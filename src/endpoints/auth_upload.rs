use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::util::animalnumbers::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;
use crate::i18n::filters;

#[derive(Template)]
#[template(path = "auth_upload.html")]
struct AuthPasta<'a> {
    args: &'a Args,
    id: String,
    status: String,
    /// 本次要求输入的是哪一项权限的密码：`read` 或 `write`。
    permission: String,
    path: String,
    /// 追加到表单 action 之后的查询串（含 `?`，没有则为空串）。
    ///
    /// 密码提交后仍要落到原来的目标上，因此需要把它原样带过去：
    /// * `/secure_file/{id}?fname=...` —— 指定要下载的附件；
    /// * `/archive/{id}` —— 打包下载全部附件。
    query: String,
}

#[get("/auth/{id}")]
pub async fn auth_upload(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    permission: String::from("read"),
                    path: String::from("upload"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth/{id}/{status}")]
pub async fn auth_upload_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    permission: String::from("read"),
                    path: String::from("upload"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_raw/{id}")]
pub async fn auth_raw_pasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    permission: String::from("read"),
                    path: String::from("raw"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_raw/{id}/{status}")]
pub async fn auth_raw_pasta_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    permission: String::from("read"),
                    path: String::from("raw"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_edit_private/{id}")]
pub async fn auth_edit_private(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    permission: String::from("read"),
                    path: String::from("edit_private"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_edit_private/{id}/{status}")]
pub async fn auth_edit_private_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    permission: String::from("read"),
                    path: String::from("edit_private"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

/// `/auth_file/{id}` 与 `/auth_file/{id}/{status}` 共用的实现。
///
/// 请求上的查询串有两个用途：
/// * `to=archive` —— 密码提交后走「打包下载」(`/archive/{id}`)，
///   而不是「下载单个文件」(`/secure_file/{id}`)；
/// * `fname=...` —— 指定要下载的哪个附件，提交时原样带回 `/secure_file`。
fn auth_file_page(
    data: web::Data<AppState>,
    id: String,
    status: String,
    query: &str,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    let to_archive = query.split('&').any(|pair| pair == "to=archive");

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    permission: String::from("read"),
                    path: String::from(if to_archive { "archive" } else { "secure_file" }),
                    query: if query.is_empty() {
                        String::new()
                    } else {
                        format!("?{}", query)
                    },
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_file/{id}")]
pub async fn auth_file(
    data: web::Data<AppState>,
    id: web::Path<String>,
    request: actix_web::HttpRequest,
) -> HttpResponse {
    auth_file_page(
        data,
        id.into_inner(),
        String::from(""),
        request.query_string(),
    )
}

#[get("/auth_file/{id}/{status}")]
pub async fn auth_file_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
    request: actix_web::HttpRequest,
) -> HttpResponse {
    let (id, status) = param.into_inner();
    auth_file_page(data, id, status, request.query_string())
}

#[get("/auth_remove_private/{id}")]
pub async fn auth_remove_private(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    permission: String::from("write"),
                    path: String::from("remove"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_remove_private/{id}/{status}")]
pub async fn auth_remove_private_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    permission: String::from("write"),
                    path: String::from("remove"),
                    query: String::new(),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

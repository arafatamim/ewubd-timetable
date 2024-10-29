use actix_web::{get, http, post, web, HttpRequest, HttpResponse, Responder};
use maud::html;
use serde::{Deserialize, Serialize};

use crate::partials::page;
use ewubd_timetable_calendar_lib::{auth, utils};

#[get("/")]
pub async fn index(req: HttpRequest) -> impl Responder {
    if utils::get_session_cookie(&req).is_ok() {
        return HttpResponse::build(http::StatusCode::FOUND)
            .append_header((http::header::LOCATION, "/dashboard"))
            .finish();
    }

    let login_page = page(
        "Login",
        false,
        html! {
            form action="/" method="post" {
                input type="text" name="username" placeholder="Student ID" required;
                br;
                input type="password" name="password" placeholder="Password" required;
                br;
                input type="submit" value="Login";
            }
        },
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(login_page.into_string())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[post("/")]
pub async fn login(form: web::Form<LoginData>) -> impl Responder {
    let login_page_res = auth::fetch_login_page().await?;
    let session_id = auth::get_session_id(&login_page_res)?;
    let login_page_html = login_page_res
        .text()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let (first_num, second_num) = auth::get_captcha_addends(&login_page_html)?;

    let client = utils::build_authenticated_client(&session_id)?;

    auth::authenticate(
        &client,
        &form.username,
        &form.password,
        first_num,
        second_num,
    )
    .await
    .map_err(|_| actix_web::error::ErrorUnauthorized("Authorization failed"))?;

    let mut cookie = String::new();
    cookie.push_str(&session_id);
    cookie.push_str("; max-age=900");

    let response = HttpResponse::build(http::StatusCode::FOUND)
        .append_header((http::header::LOCATION, "/dashboard"))
        .append_header((http::header::SET_COOKIE, session_id))
        .finish();

    Ok::<HttpResponse, actix_web::error::Error>(response)
}

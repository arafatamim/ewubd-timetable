use actix_web::{error, get, HttpRequest, HttpResponse};
use ewubd_timetable_calendar_lib::utils;

#[get("/logout")]
pub async fn logout(req: HttpRequest) -> Result<HttpResponse, error::Error> {
    if utils::get_session_cookie(&req).is_ok() {
        let mut response = HttpResponse::build(actix_web::http::StatusCode::FOUND);
        response.append_header((actix_web::http::header::LOCATION, "/"));
        response.append_header((
            actix_web::http::header::SET_COOKIE,
            "ASP.NET_SessionId=; Max-Age=-1; HttpOnly",
        ));
        Ok(response.finish())
    } else {
        Ok(HttpResponse::build(actix_web::http::StatusCode::UNAUTHORIZED).finish())
    }
}

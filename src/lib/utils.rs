use reqwest::header;
use std::error::Error;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3";

pub fn build_headers(session_id: &str) -> header::HeaderMap {
    use header::{HeaderMap, HeaderValue};

    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    headers.insert(header::COOKIE, HeaderValue::from_str(session_id).unwrap());
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_str("https://portal.ewubd.edu").unwrap(),
    );
    headers.insert(
        header::REFERER,
        HeaderValue::from_str("https://portal.ewubd.edu/").unwrap(),
    );
    headers
}

pub fn build_authenticated_client(session_id: &str) -> Result<reqwest::Client, Box<dyn Error>> {
    let headers = build_headers(session_id);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    Ok(client)
}

pub fn get_session_cookie(req: &actix_web::HttpRequest) -> Result<String, actix_web::error::Error> {
    let cookies = req.cookies();
    let cookie = cookies
        .iter()
        .find_map(|c| c.iter().find(|c| c.name() == "ASP.NET_SessionId"));

    match cookie {
        Some(cookie) => Ok(cookie.to_string()),
        None => Err(actix_web::error::ErrorUnauthorized("Session cookie not found")),
    }
}


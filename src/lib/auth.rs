use scraper::{Html, Selector};
use std::error::Error;

use crate::utils::USER_AGENT;
use reqwest::{
    header::{self},
    Response,
};

pub async fn fetch_login_page() -> Result<Response, Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::USER_AGENT,
                header::HeaderValue::from_static(USER_AGENT),
            );
            headers
        })
        .build()?;

    let login_page_res = client.get("https://portal.ewubd.edu/").send().await?;
    Ok(login_page_res)
}

pub fn get_session_id(login_page_res: &Response) -> Result<String, Box<dyn Error>> {
    let session_id = login_page_res
        .headers()
        .get("set-cookie")
        .ok_or("Cookie not found")?
        .to_str()?
        .split(";")
        .next()
        .ok_or("Invalid session cookie format")?
        .to_string();

    Ok(session_id)
}

pub fn get_captcha_addends(login_page_html: &str) -> Result<(i8, i8), Box<dyn Error>> {
    let doc = Html::parse_document(login_page_html);
    let first_num_selector = Selector::parse("[name=FirstNo]")?;
    let first_num = doc
        .select(&first_num_selector)
        .next()
        .and_then(|v| v.attr("value"))
        .and_then(|v| v.parse::<i8>().ok())
        .ok_or("Invalid first number")?;
    let second_num_selector = Selector::parse("[name=SecondNo]")?;
    let second_num = doc
        .select(&second_num_selector)
        .next()
        .and_then(|v| v.attr("value"))
        .and_then(|v| v.parse::<i8>().ok())
        .ok_or("Invalid second number")?;

    Ok((first_num, second_num))
}

pub async fn authenticate<'a>(
    client: &reqwest::Client,
    username: &'a str,
    password: &'a str,
    first_num: i8,
    second_num: i8,
) -> Result<String, Box<dyn Error>> {
    let res = client
        .post("https://portal.ewubd.edu")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(format!(
            "Username={}&Password={}&FirstNo={}&SecondNo={}&Answer={}",
            username,
            password,
            first_num,
            second_num,
            first_num + second_num
        ))
        .send()
        .await?;

    if res.status().is_success() || res.status().is_redirection() {
        let html = res.text().await?;
        let doc = Html::parse_document(&html);

        let error_msg_selector = Selector::parse(".error")?;
        let error_msg = doc
            .select(&error_msg_selector)
            .next()
            .map(|v| v.text().collect::<String>());

        match error_msg {
            Some(msg) => Err(msg.into()),
            _ => {
                let welcome_msg_selector = Selector::parse(".nav-user > span")?;
                let welcome_msg = doc
                    .select(&welcome_msg_selector)
                    .next()
                    .map(|v| v.text().collect::<String>())
                    .ok_or("Welcome msg not found")?;

                Ok(welcome_msg)
            }
        }
    } else {
        Err(res.text().await?.into())
    }
}

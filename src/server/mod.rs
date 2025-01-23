use std::sync::Mutex;

use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    middleware::{from_fn, Logger, Next},
    web::Data,
    App, HttpServer,
};
use env_logger::Env;

mod partials;
mod routes;

use maud::html;
use partials::page;

#[derive(Debug, Hash, PartialEq)]
pub struct CalendarEntry {
    pub created_at: std::time::Instant,
    pub ical: String,
}

struct AppState {
    /// Stores a hashmap of temporary generated calendars
    calendars: Mutex<std::collections::HashMap<String, CalendarEntry>>,
}

/// Middleware to delete calendars that are older than 15 minutes
async fn cleanup_calendars(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    {
        let state =
            req.app_data::<Data<AppState>>()
                .ok_or(actix_web::error::ErrorInternalServerError(
                    "Cannot access app state",
                ))?;

        let mut calendars = state
            .calendars
            .lock()
            .map_err(|_| actix_web::error::ErrorInternalServerError("Cannot access calendars"))?;
        calendars.retain(|_, entry| entry.created_at.elapsed().as_secs() < 900);
    }

    next.call(req).await
}

async fn auth_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    let res = next.call(req).await;

    res.map(|mut res| {
        if res.status() == StatusCode::UNAUTHORIZED {
            let login_page = page(
                "Login",
                false,
                html! {
                    p {
                        mark { "Please login again. Possible errors: session expired, invalid credentials, or login blocked." };
                    }
                    form action="/" method="post" {
                        input type="text" name="username" placeholder="Student ID";
                        input type="password" name="password" placeholder="Password";
                        input type="submit" value="Login";
                    }
                },
            );

            res.headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
            res.map_body(|_, _| BoxBody::new(login_page.into_string()))
        } else {
            res.map_into_boxed_body()
        }
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use routes::*;

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let app_data = Data::new(AppState {
        calendars: Mutex::new(std::collections::HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(from_fn(auth_middleware))
            .wrap(from_fn(cleanup_calendars))
            .service(index::index)
            .service(index::login)
            .service(dashboard::dashboard)
            .service(dashboard::timetable)
            .service(dashboard::generate)
            .service(dashboard::download)
            .service(logout::logout)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}

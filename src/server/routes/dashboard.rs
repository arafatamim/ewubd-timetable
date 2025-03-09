use std::collections::HashMap;

use actix_web::{error, get, http, post, web, HttpRequest, HttpResponse};
use chrono::NaiveDate;
use ewubd_timetable_calendar_lib::{calendar, courses, periods, semester, utils};
use maud::{html, Markup};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{partials::page, AppState, CalendarEntry};

#[get("/dashboard")]
pub async fn dashboard(req: HttpRequest) -> Result<Markup, error::Error> {
    let session_cookie = utils::get_session_cookie(&req)?;

    // TODO: check if session is valid

    let client = utils::build_authenticated_client(&session_cookie)?;

    let semesters = semester::get_all_semesters(&client).await.map_err(|e| {
        match e.downcast_ref::<reqwest::Error>().and_then(|e| e.status()) {
            Some(StatusCode::UNAUTHORIZED) => error::ErrorUnauthorized("Unauthorized"),
            _ => error::ErrorInternalServerError("Internal server error"),
        }
    })?;

    let markup = page(
        "Welcome!",
        true,
        html! {
            div {
                h2 { "Fetch timetable" }
                form action="/dashboard/timetable" method="post" {
                    label for="semester" { "Select Semester" };
                    select id="semester" name="semester" {
                        @for semester in semesters {
                            option value=(format!("{} {}", semester.id, semester.name)) { (semester.name) }
                        }
                    };
                    br;
                    input type="submit" value="Generate";
                }
            }
        },
    );

    Ok(markup)
}

#[derive(Deserialize)]
struct TimetableForm {
    semester: String,
}

#[post("/dashboard/timetable")]
pub async fn timetable(
    req: HttpRequest,
    form: web::Form<TimetableForm>,
) -> Result<Markup, error::Error> {
    let session_cookie = utils::get_session_cookie(&req)?;

    let client = utils::build_authenticated_client(&session_cookie)?;

    let mut semester_param = form.semester.split(" ");
    let semester_id = semester_param.next().unwrap().parse::<u16>().unwrap();
    let semester_name = semester_param.next().unwrap();

    let courses = courses::get_courses(&client, semester_id)
        .await
        .map_err(
            |e| match e.downcast_ref::<reqwest::Error>().and_then(|e| e.status()) {
                Some(StatusCode::UNAUTHORIZED) => error::ErrorUnauthorized("Unauthorized"),
                _ => error::ErrorInternalServerError("Internal server error"),
            },
        )?;

    let body = page(
        &format!("Timetable for Semester {}", form.semester),
        true,
        html! {
            table {
                tr {
                    th { "Course Code" }
                    th { "Section" }
                    th { "Lecturer" }
                    th { "Periods" }
                }
                @for course in &courses {
                    tr {
                        td { (course.course_code) }
                        td { (course.section) }
                        td { (course.lecturer) }
                        td {
                            ul {
                                @for period in &course.periods {
                                    li { (periods::Period::weekday_name(period.day)) " " (period.start_time)"–"(period.end_time) " @ " (period.room) }
                                }
                            }
                        }
                    }
                }
            }
            article {
                h2 { "Generate timetable calendar" }
                form action="/dashboard/timetable/generate" method="post" {
                    input type="hidden" name="semester_id" value=(semester_id);
                    label for="semester_name" { "Semester Name" };
                    input type="text" name="semester_name" value=(semester_name.replace("-", " "));
                    br;
                    label for="start_date" { "Semester Start Date" };
                    input type="date" name="start_date" placeholder="Start Date" required;
                    br;
                    label for="end_date" { "Semester End Date" };
                    input type="date" name="end_date" placeholder="End Date" required;
                    br;
                    input type="submit" value="Generate Calendar";
                }
            }
        },
    );

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct GenerateForm {
    semester_id: u16,
    semester_name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[post("/dashboard/timetable/generate")]
pub async fn generate(
    req: HttpRequest,
    form: web::Form<GenerateForm>,
    state: web::Data<AppState>,
) -> Result<Markup, error::Error> {
    let session_cookie = utils::get_session_cookie(&req)?;

    let client = utils::build_authenticated_client(&session_cookie)?;

    let GenerateForm {
        semester_id,
        semester_name,
        start_date,
        end_date,
    } = form.into_inner();

    let courses = courses::get_courses(&client, semester_id)
        .await
        .map_err(
            |e| match e.downcast_ref::<reqwest::Error>().and_then(|e| e.status()) {
                Some(StatusCode::UNAUTHORIZED) => error::ErrorUnauthorized("Unauthorized"),
                _ => error::ErrorInternalServerError("Internal server error"),
            },
        )?;

    let ical = calendar::build_timetable(courses, &semester_name, start_date, end_date)?;

    let id = xxhash_rust::xxh3::xxh3_64(
        format!("{session_cookie}{semester_id}{semester_name}").as_bytes(),
    )
    .to_string();

    let mut calendars = state
        .calendars
        .lock()
        .map_err(|_| error::ErrorInternalServerError("Cannot access calendars"))?;

    calendars.insert(
        id.clone(),
        CalendarEntry {
            created_at: std::time::Instant::now(),
            ical,
        },
    );

    let host = req.connection_info().host().to_string();
    let calendar_path = format!("/dashboard/timetable/download?id={}", id);

    Ok(page(
        "Calendar Generated",
        true,
        html! {
            p { "Calendar for timetable generated successfully." }
            article {
                "Semester ID: " (semester_id); br;
                "Semester Name: " (semester_name); br;
                "Start Date: " (start_date); br;
                "End Date: " (end_date)
            }
            p { "Note: Links will expire after 15 minutes" }
            p {
                a href=(format!("https://calendar.google.com/calendar/u/0/r?cid=webcal://{host}{calendar_path}")) target="_blank" { "Add to Google Calendar" }
            }
            p {
                a href=(format!("/dashboard/timetable/download?id={}", id)) { "Download as iCal" }
            }
        },
    ))
}

#[get("/dashboard/timetable/download")]
pub async fn download(
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, error::Error> {
    let id = query
        .get("id")
        .ok_or(error::ErrorBadRequest("Missing id"))?;

    let calendars = state
        .calendars
        .lock()
        .map_err(|_| error::ErrorInternalServerError("Cannot access calendars"))?;

    let entry = calendars
        .get(id)
        .ok_or(error::ErrorNotFound("Calendar doesn't exist"))?;

    Ok(HttpResponse::Ok()
        .content_type("text/calendar")
        .insert_header((http::header::CACHE_CONTROL, "public, max-age=900"))
        .insert_header((
            http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=timetable_{}.ics", id),
        ))
        .body(entry.ical.to_string()))
}

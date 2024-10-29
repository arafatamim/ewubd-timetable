use std::error::Error;

use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let login_page_res = auth::fetch_login_page().await?;
    // let session_id = auth::get_session_id(&login_page_res)?;
    // let login_page_html = login_page_res.text().await?;
    // let (first_num, second_num) = auth::get_captcha_addends(&login_page_html)?;
    //
    // let client = build_authenticated_client(&session_id)?;
    //
    // let welcome_msg = auth::authenticate(
    //     &client,
    //     "2020-1-77-038",
    //     "XUyjVB3M4e*J*fhg",
    //     first_num,
    //     second_num,
    // )
    // .await?;
    //
    // println!("{}", welcome_msg);
    //
    // let courses_json = courses::fetch_courses_as_json(&client, 135).await?;
    // // std::fs::read_to_string("courses.json").expect("Unable to read courses.json");
    //
    // let courses = courses::parse_courses(courses_json).unwrap();
    //
    // let calendar = calendar::build_timetable(
    //     courses,
    //     "Fall 2024 Timetable",
    //     NaiveDate::from_ymd_opt(2024, 10, 20).unwrap(),
    //     NaiveDate::from_ymd_opt(2025, 2, 20).unwrap(),
    // )
    // .unwrap();
    //
    // println!("{}", calendar);

    Ok(())
}

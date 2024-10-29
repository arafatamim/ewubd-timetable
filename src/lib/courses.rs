use reqwest::Client;
use std::error::Error;

use crate::periods::Period;

#[derive(Debug)]
pub struct Course {
    pub course_code: String,
    pub section: u8,
    pub lecturer: String,
    pub periods: Vec<Period>,
}

pub async fn fetch_courses_as_json(
    client: &Client,
    semester_id: u16,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let res = client
        .get(format!(
        "https://portal.ewubd.edu/api/Advising/GetSemesterStudentWiseAdvisingCourseListStudent/{}",
        semester_id
    ))
        .send()
        .await?
        .error_for_status()?;
    let json = res.json::<serde_json::Value>().await?;

    Ok(json)
}

pub fn parse_courses(courses_json: serde_json::Value) -> Result<Vec<Course>, Box<dyn Error>> {
    let courses_json_array = courses_json
        .as_array()
        .ok_or("Invalid course JSON response")?;

    let mut parsed_courses = Vec::<Course>::new();

    for course_json in courses_json_array {
        let dropped = course_json["DropStatus"]
            .as_str()
            .ok_or("Invalid drop status")?
            .to_lowercase()
            == "yes";
        let withdrawn = course_json["WithDrawStatus"]
            .as_str()
            .ok_or("Invalid withdrawn status")?
            .to_lowercase()
            == "yes";

        if dropped || withdrawn {
            continue;
        }

        let time_slot_name = course_json["TimeSlotName"]
            .as_str()
            .ok_or("Invalid time slot name")?;
        let room_name = course_json["RoomName"]
            .as_str()
            .ok_or("Invalid room name")?;
        let course_code = course_json["CourseCode"]
            .as_str()
            .ok_or("Invalid course code")?;
        let section_name = course_json["SectionName"]
            .as_i64()
            .ok_or("Invalid section")? as u8;
        let faculty_name = course_json["FacultyName"]
            .as_str()
            .ok_or("Invalid faculty name")?;

        let mut periods = Period::parse_periods(time_slot_name, room_name)?;

        let existing_course = parsed_courses
            .iter_mut()
            .find(|c| c.course_code == course_code && c.section == section_name);

        match existing_course {
            None => {
                let course = Course {
                    course_code: course_code.to_string(),
                    section: section_name,
                    lecturer: faculty_name.to_string(),
                    periods,
                };
                parsed_courses.push(course);
            }
            Some(course) => {
                // merge the time slots if the course already exists
                course.periods.append(&mut periods);
            }
        }
    }

    Ok(parsed_courses)
}

pub async fn get_courses(client: &Client, semester_id: u16) -> Result<Vec<Course>, Box<dyn Error>> {
    let courses_json = fetch_courses_as_json(client, semester_id).await?;
    let courses = parse_courses(courses_json)?;

    Ok(courses)
}

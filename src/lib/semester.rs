use chrono::{NaiveDate, NaiveDateTime};
use reqwest::Client;
use std::error::Error;
use std::result::Result;

#[derive(Debug)]
pub struct Semester {
    pub id: u16,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

pub async fn get_all_semesters(
    client: &Client,
) -> Result<Vec<Semester>, Box<dyn Error>> {
    let res = client
        .get("https://portal.ewubd.edu/api/utility/GetSemesterForDropDown")
        .send()
        .await?
        .error_for_status()?;

    let json = res.json::<serde_json::Value>().await?;

    let array = json.as_array().ok_or("Invalid semester JSON response")?;

    let mut semesters = Vec::<Semester>::new();

    for item in array {
        let id = item["SemesterId"].as_u64().ok_or("Invalid semester ID")? as u16;
        let name = item["SemesterName"]
            .as_str()
            .ok_or("Invalid semester name")?
            .to_string();
        let start_date = item["StartingDate"]
            .as_str()
            .ok_or("Invalid start date")
            .map(|s| s.parse::<NaiveDateTime>())??;
        let end_date = item["EndingDate"]
            .as_str()
            .ok_or("Invalid end date")
            .map(|s| s.parse::<NaiveDateTime>())??;

        semesters.push(Semester {
            id,
            name,
            start_date: start_date.date(),
            end_date: end_date.date(),
        });
    }

    Ok(semesters)
}

use chrono::{Datelike, NaiveDate, NaiveTime, Utc, Weekday};
use ics::{
    components::{Parameter, Property},
    properties::{CalScale, Description, DtEnd, DtStart, Location, Method, Name, RRule, Summary},
    Event, ICalendar, Standard, TimeZone as ICSTimeZone,
};
use std::error::Error;

use crate::courses::Course;

pub fn find_first_weekday(start_date: NaiveDate, day_index: u8) -> Option<NaiveDate> {
    let target_weekday = match day_index {
        0 => Weekday::Sun,
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        _ => return None,
    };

    let current_weekday = start_date.weekday();

    let days_until =
        (7 + target_weekday.num_days_from_sunday() - current_weekday.num_days_from_sunday()) % 7;

    start_date.checked_add_signed(chrono::Duration::days(days_until as i64))
}

fn get_two_letter_weekday(day_index: u8) -> &'static str {
    match day_index {
        0 => "SU",
        1 => "MO",
        2 => "TU",
        3 => "WE",
        4 => "TH",
        5 => "FR",
        6 => "SA",
        _ => panic!("Invalid day index"),
    }
}

pub fn build_timetable(
    courses: Vec<Course>,
    name: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<String, Box<dyn Error>> {
    let mut calendar = ICalendar::new("2.0", format!("-//East West University//{}//EN", name));

    let timezone = ICSTimeZone::standard(
        "Asia/Dhaka",
        Standard::new("19700101T000000", "+0600", "+0600"),
    );
    calendar.add_timezone(timezone);
    calendar.push(Name::new(name));
    calendar.push(Property::new("X-WR-CALNAME", name));
    calendar.push(CalScale::new("GREGORIAN"));
    calendar.push(Method::new("PUBLISH"));

    for course in courses {
        for period in course.periods {
            let ev_hash = xxhash_rust::xxh3::xxh3_64(
                format!(
                    "{}{}{}",
                    course.course_code.clone(),
                    period.room.clone(),
                    period.day
                )
                .as_bytes(),
            );

            let mut event = Event::new(
                format!("{:x}", ev_hash),
                Utc::now().format("%Y%m%dT000000").to_string(),
            );

            let course_start_date = find_first_weekday(start_date, period.day)
                .ok_or("Error finding first date of course")?;

            let period_start = NaiveTime::from_hms_opt(
                period.start_time.hours as u32,
                period.start_time.minutes as u32,
                0,
            )
            .ok_or("Invalid period start time")?;
            let period_end = NaiveTime::from_hms_opt(
                period.end_time.hours as u32,
                period.end_time.minutes as u32,
                0,
            )
            .ok_or("Invalid period end time")?;

            let mut dtstart = DtStart::new(
                course_start_date
                    .and_time(period_start)
                    .format("%Y%m%dT%H%M%S")
                    .to_string(),
            );
            dtstart.add(Parameter::new("TZID", "Asia/Dhaka"));

            let mut dtend = DtEnd::new(
                course_start_date
                    .and_time(period_end)
                    .format("%Y%m%dT%H%M%S")
                    .to_string(),
            );
            dtend.add(Parameter::new("TZID", "Asia/Dhaka"));

            let mut rrule = RRule::new("FREQ=WEEKLY");
            rrule.add(Parameter::new("BYDAY", get_two_letter_weekday(period.day)));
            rrule.add(Parameter::new(
                "UNTIL",
                end_date.format("%Y%m%d").to_string(),
            ));

            event.push(Summary::new(format!(
                "{} ({})",
                course.course_code.clone(),
                course.section
            )));
            event.push(Location::new(period.room.clone()));
            event.push(Description::new(format!(
                "Lecturer: {}",
                course.lecturer.clone()
            )));
            event.push(dtstart);
            event.push(dtend);
            event.push(rrule);

            calendar.add_event(event);
        }
    }

    Ok(calendar.to_string())
}

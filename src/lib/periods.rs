use std::{error::Error, fmt::Display};

/// Stores 24-hr time
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
}

impl Time {
    pub fn new(hours: u8, minutes: u8) -> Self {
        Time { hours, minutes }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let period = if self.hours < 12 { "AM" } else { "PM" };
        let hours = if self.hours == 0 || self.hours == 12 {
            12
        } else {
            self.hours % 12
        };
        write!(f, "{:02}:{:02}{}", hours, self.minutes, period)
    }
}

impl TryFrom<&str> for Time {
    type Error = Box<dyn Error>;

    /// Parse a 12-hr time string in the format "HH:MMAM" or "HH:MMPM" into 24-hr
    fn try_from(time: &str) -> Result<Time, Box<dyn Error>> {
        let re = regex::Regex::new(r"(\d{1,2}):(\d{2})(AM|PM)")?;
        let caps = re.captures(time).ok_or("Invalid time string")?;
        let hours = caps.get(1).ok_or("Invalid hours")?.as_str().parse::<u8>()?;
        let minutes = caps
            .get(2)
            .ok_or("Invalid minutes")?
            .as_str()
            .parse::<u8>()?;
        let period = caps.get(3).ok_or("AM/PM not present")?.as_str();

        let hours = match period {
            "AM" => {
                if hours == 12 {
                    0
                } else {
                    hours
                }
            }
            "PM" => {
                if hours == 12 {
                    hours
                } else {
                    hours + 12
                }
            }
            _ => panic!("Invalid period"),
        };

        Ok(Time { hours, minutes })
    }
}

/// Stores a time slot
#[derive(Debug, PartialEq, Clone)]
pub struct Period {
    /// Stores an index from 0 to 6, where 0 is Sunday and 6 is Saturday
    pub day: u8,
    pub start_time: Time,
    pub end_time: Time,
    pub room: String,
}

impl Period {
    pub fn parse_weekday_letter(day: char) -> u8 {
        match day {
            'S' => 0,
            'M' => 1,
            'T' => 2,
            'W' => 3,
            'R' => 4,
            'F' => 5,
            'A' => 6,
            _ => panic!("Invalid day letter"),
        }
    }

    pub fn weekday_name(day: u8) -> String {
        match day {
            0 => "Sunday".to_string(),
            1 => "Monday".to_string(),
            2 => "Tuesday".to_string(),
            3 => "Wednesday".to_string(),
            4 => "Thursday".to_string(),
            5 => "Friday".to_string(),
            6 => "Saturday".to_string(),
            _ => panic!("Invalid day index"),
        }
    }

    pub fn parse_periods(value: &str, room: &str) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut split_str = value.split(" ");
        let day_letters = split_str.next().ok_or("Garbage input")?;
        let time_range = split_str.next().ok_or("Garbage input")?;

        let days = day_letters.chars().map(Period::parse_weekday_letter);

        let mut time_range = time_range.split("-");
        let start_time: Time = time_range.next().ok_or("Invalid start time")?.try_into()?;
        let end_time: Time = time_range.next().ok_or("Invalid time slot")?.try_into()?;

        Ok(days
            .map(|day| Period {
                day,
                start_time,
                end_time,
                room: room.to_string(),
            })
            .collect::<Vec<Period>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_from() {
        let time = Time::try_from("12:00AM").unwrap();
        assert_eq!(time.hours, 0);
        assert_eq!(time.minutes, 0);

        let time = Time::try_from("12:00PM").unwrap();
        assert_eq!(time.hours, 12);
        assert_eq!(time.minutes, 0);

        let time = Time::try_from("1:00AM").unwrap();
        assert_eq!(time.hours, 1);
        assert_eq!(time.minutes, 0);

        let time = Time::try_from("1:00PM").unwrap();
        assert_eq!(time.hours, 13);
        assert_eq!(time.minutes, 0);

        let time = Time::try_from("6:30PM").unwrap();
        assert_eq!(time.hours, 18);
        assert_eq!(time.minutes, 30);
    }

    #[test]
    fn parse_time_slots() {
        let period = Period::parse_periods("MW 9:25AM-10:40AM", "Room 1").unwrap();

        assert_eq!(
            period,
            vec![
                Period {
                    day: 1,
                    start_time: Time::new(9, 25),
                    end_time: Time::new(10, 40),
                    room: "Room 1".to_string(),
                },
                Period {
                    day: 3,
                    start_time: Time::new(9, 25),
                    end_time: Time::new(10, 40),
                    room: "Room 1".to_string(),
                }
            ]
        );
    }
}

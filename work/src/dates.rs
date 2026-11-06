use chrono::{
    Days, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Timelike, Utc,
};

pub(crate) fn parse(text: &str) -> Option<NaiveDate> {
    let bytes = text.as_bytes();
    if bytes.len() != 10
        || bytes.iter().enumerate().any(|(index, byte)| {
            if matches!(index, 4 | 7) {
                *byte != b'-'
            } else {
                !byte.is_ascii_digit()
            }
        })
    {
        return None;
    }
    NaiveDate::parse_from_str(text, "%Y-%m-%d").ok()
}

pub(crate) struct Period {
    pub(crate) first_day: NaiveDate,
    pub(crate) start: i64,
    pub(crate) end: i64,
}

/// A date written `YYYY-MM-DD`, or a date and time written `YYYY-MM-DD HH:MM`: the two forms
/// of `checked`. A date alone is 00:00 that day.
pub(crate) fn parse_with_time(text: &str) -> Option<NaiveDateTime> {
    let (date, time) = match text.split_once(' ') {
        Some((date, time)) => (date, Some(time)),
        None => (text, None),
    };
    let date = parse(date)?;
    let time = match time {
        None => NaiveTime::MIN,
        Some(time) => {
            let bytes = time.as_bytes();
            if bytes.len() != 5
                || bytes[2] != b':'
                || ![0, 1, 3, 4]
                    .iter()
                    .all(|&index| bytes[index].is_ascii_digit())
            {
                return None;
            }
            NaiveTime::parse_from_str(time, "%H:%M").ok()?
        }
    };
    Some(date.and_time(time))
}

/// Now to the minute, in the local time zone, or the date and time `ROOSTR_TODAY` names.
pub(crate) fn now() -> Result<NaiveDateTime, String> {
    match std::env::var("ROOSTR_TODAY") {
        Ok(text) => parse_with_time(&text).ok_or_else(|| {
            format!(
                "Invalid ROOSTR_TODAY {text:?}; expected a date written YYYY-MM-DD or a date \
                 and time written YYYY-MM-DD HH:MM"
            )
        }),
        Err(std::env::VarError::NotPresent) => {
            let now = Local::now().naive_local();
            Ok(now
                .with_second(0)
                .and_then(|now| now.with_nanosecond(0))
                .unwrap_or(now))
        }
        Err(error) => Err(format!("Cannot read ROOSTR_TODAY: {error}")),
    }
}

/// Now in seconds since the epoch, or the date and time `ROOSTR_TODAY` names, in the local
/// time zone.
pub(crate) fn now_seconds() -> Result<i64, String> {
    if std::env::var_os("ROOSTR_TODAY").is_none() {
        return Ok(Utc::now().timestamp());
    }
    let now = now()?;
    Local
        .from_local_datetime(&now)
        .earliest()
        .map(|now| now.timestamp())
        .ok_or_else(|| format!("{now} does not exist in the local time zone"))
}

/// Today in the local time zone, or the date `ROOSTR_TODAY` names.
pub(crate) fn today() -> Result<NaiveDate, String> {
    Ok(now()?.date())
}

/// A value for `checked` that differs from every one in `held`, so that writing it is a
/// change: today's date alone when none holds today's date; otherwise today's date and the
/// time `now`, or the first minute after it that none holds.
pub(crate) fn checked_value<'a>(
    now: NaiveDateTime,
    held: impl IntoIterator<Item = &'a str>,
) -> String {
    let held: Vec<&str> = held.into_iter().collect();
    let today = now.date().to_string();
    let holds_today = held
        .iter()
        .any(|value| parse_with_time(value).is_some_and(|written| written.date() == now.date()));
    if !holds_today {
        return today;
    }
    let mut value = now;
    loop {
        let text = value.format("%Y-%m-%d %H:%M").to_string();
        if !held.contains(&text.as_str()) {
            return text;
        }
        value += TimeDelta::minutes(1);
    }
}

impl Period {
    pub(crate) fn since(when: &str) -> Result<Self, String> {
        let today = today()?;
        let first_day = first_day(when, today).ok_or_else(|| {
            format!(
                "Invalid --since value {when:?}; expected days like 7d or a date written YYYY-MM-DD"
            )
        })?;
        let midnight = first_day.and_time(NaiveTime::MIN);
        let start = Local
            .from_local_datetime(&midnight)
            .earliest()
            .ok_or_else(|| {
                format!("Midnight on {first_day} does not exist in the local time zone")
            })?
            .timestamp();
        Ok(Self {
            first_day,
            start,
            end: Utc::now().timestamp(),
        })
    }

    pub(crate) fn contains(&self, timestamp: i64) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
}

fn first_day(when: &str, today: NaiveDate) -> Option<NaiveDate> {
    if let Some(days) = when.strip_suffix('d') {
        if days.is_empty() || !days.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        today.checked_sub_days(Days::new(days.parse().ok()?))
    } else {
        parse(when)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_require_the_exact_form_and_a_real_calendar_day() {
        for text in [
            "yesterday",
            "2026-02-30",
            "1900-02-29",
            "2026-2-03",
            "2026-02-3",
            "2026-02-03\n",
        ] {
            assert!(parse(text).is_none(), "{text:?}");
        }
        for text in ["2000-02-29", "2024-02-29", "2026-12-31"] {
            assert!(parse(text).is_some(), "{text}");
        }
    }

    #[test]
    fn checked_takes_a_date_or_a_date_and_a_time_to_the_minute() {
        for text in [
            "2026-10-10",
            "2026-10-10 14:30",
            "2026-10-10 00:00",
            "2026-10-10 23:59",
        ] {
            assert!(parse_with_time(text).is_some(), "{text}");
        }
        for text in [
            "2026-10-10T14:30",
            "2026-10-10 24:00",
            "2026-10-10 14:30:15",
            "2026-10-10 9:30",
            "2026-10-10  14:30",
            "2026-10-10 14:60",
            "2026-10-10 +1:30",
        ] {
            assert!(parse_with_time(text).is_none(), "{text}");
        }
    }

    #[test]
    fn the_suggested_checked_value_differs_from_every_one_held() {
        let at = |text| parse_with_time(text).unwrap();
        let now = at("2026-10-10 14:30");
        assert_eq!(checked_value(now, ["2026-10-08"]), "2026-10-10");
        assert_eq!(
            checked_value(now, ["2026-10-10", "2026-10-08"]),
            "2026-10-10 14:30"
        );
        assert_eq!(checked_value(now, ["2026-10-10 09:15"]), "2026-10-10 14:30");
        assert_eq!(
            checked_value(now, ["2026-10-10 14:30", "2026-10-10 14:31"]),
            "2026-10-10 14:32"
        );
        assert_eq!(
            checked_value(at("2026-10-10 23:59"), ["2026-10-10 23:59"]),
            "2026-10-11 00:00"
        );
        assert_eq!(
            checked_value(at("2026-10-10"), ["2026-10-10"]),
            "2026-10-10 00:00"
        );
    }

    #[test]
    fn relative_days_cross_month_year_and_leap_day_boundaries() {
        for (today, since, expected) in [
            ("2027-01-02", "7d", "2026-12-26"),
            ("2024-03-01", "1d", "2024-02-29"),
            ("2026-03-01", "1d", "2026-02-28"),
            ("2026-10-10", "0d", "2026-10-10"),
        ] {
            assert_eq!(first_day(since, parse(today).unwrap()), parse(expected));
        }
        for invalid in ["14", "-7d", "+7d", "d", "18446744073709551616d"] {
            assert!(first_day(invalid, parse("2026-10-10").unwrap()).is_none());
        }
    }
}

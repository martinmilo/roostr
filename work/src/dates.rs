use chrono::{Days, Local, NaiveDate, NaiveTime, TimeZone, Utc};

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

impl Period {
    pub(crate) fn since(when: &str) -> Result<Self, String> {
        let now = Local::now();
        let today = match std::env::var("ROOSTR_TODAY") {
            Ok(text) => parse(&text).ok_or_else(|| {
                format!("Invalid ROOSTR_TODAY {text:?}; expected a date written YYYY-MM-DD")
            })?,
            Err(std::env::VarError::NotPresent) => now.date_naive(),
            Err(error) => return Err(format!("Cannot read ROOSTR_TODAY: {error}")),
        };
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

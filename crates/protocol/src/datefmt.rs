//! Date formats used in the reply frame (`dd-MM-yyyy`), without a date crate.

/// Seconds east of UTC for India Standard Time.
const IST_OFFSET_SECONDS: i64 = 19_800;

/// Seconds in one day.
const SECONDS_PER_DAY: i64 = 86_400;

/// Formats a Unix timestamp as a UTC calendar date, `dd-MM-yyyy`.
pub fn format_date_utc(unix_seconds: i64) -> String {
    let (year, month, day) = civil_from_days(unix_seconds.div_euclid(SECONDS_PER_DAY));
    format!("{day:02}-{month:02}-{year:04}")
}

/// Formats a Unix timestamp in India Standard Time, `dd-MM-yyyy HH:mm:ss`.
pub fn format_datetime_ist(unix_seconds: i64) -> String {
    let local = unix_seconds + IST_OFFSET_SECONDS;
    let (year, month, day) = civil_from_days(local.div_euclid(SECONDS_PER_DAY));
    let second_of_day = local.rem_euclid(SECONDS_PER_DAY);
    let (hour, minute, second) = (
        second_of_day / 3_600,
        second_of_day % 3_600 / 60,
        second_of_day % 60,
    );
    format!("{day:02}-{month:02}-{year:04} {hour:02}:{minute:02}:{second:02}")
}

/// Converts days since 1970-01-01 into a proleptic Gregorian (year, month,
/// day), following Howard Hinnant's `civil_from_days` algorithm.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{format_date_utc, format_datetime_ist};

    /// Formats the epoch and known dates, including a leap day.
    #[test]
    fn formats_utc_dates() {
        assert_eq!(format_date_utc(0), "01-01-1970");
        assert_eq!(format_date_utc(1_790_294_400), "25-09-2026");
        assert_eq!(format_date_utc(951_868_799), "29-02-2000");
    }

    /// Shifts by five and a half hours, across midnight and month ends.
    #[test]
    fn formats_ist_date_times() {
        assert_eq!(format_datetime_ist(1_790_294_400), "25-09-2026 05:30:00");
        assert_eq!(format_datetime_ist(951_868_799), "01-03-2000 05:29:59");
        assert_eq!(format_datetime_ist(1_834_079_399), "13-02-2028 23:59:59");
    }
}

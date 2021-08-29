use chrono::Duration;

pub const DATETIME_FMT: &str = "%Y/%m/%d %T";
pub const TIME_FMT: &str = "%T";
pub const DATE_FMT: &str = "%e-%b-%Y";

pub fn format_duration(duration: &Duration) -> String {
    format!(
        "{:02}h{:02}m",
        duration.num_hours(),
        duration.num_minutes() % 60
    )
}

use chrono::{DateTime, Utc};

// https://time-rs.github.io/api/time/format_description/well_known/struct.Rfc3339.html
pub type Time = DateTime<Utc>;
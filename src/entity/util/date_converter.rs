use chrono::{NaiveDateTime, Utc};
use tantivy::time::UtcOffset;

pub fn chrono_time_to_tantivy_datetime(chrono_dt: chrono::DateTime<chrono::Utc>) -> tantivy::DateTime {
    let timestamp_secs = chrono_dt.timestamp();
    tantivy::DateTime::from_timestamp_secs(timestamp_secs)
}
pub fn tantivy_time_to_chrono_datetime(tantivy_datetime: tantivy::DateTime) -> chrono::DateTime<Utc> {
    let offset = tantivy_datetime.into_offset(UtcOffset::UTC);

    let unix_timestamp = offset.unix_timestamp(); // Get seconds since UNIX epoch
    let nanos = offset.nanosecond(); // Get nanoseconds past the second

    // Create a NaiveDateTime from the timestamp
    let naive_datetime = NaiveDateTime::from_timestamp(unix_timestamp, nanos);

    // Convert NaiveDateTime to DateTime<Utc>
    chrono::DateTime::<Utc>::from_utc(naive_datetime, Utc)
}

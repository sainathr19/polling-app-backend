use bson::Bson;
use chrono::Utc;
pub fn get_current_time_bson() -> Bson {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    Bson::String(formatted_time)
}

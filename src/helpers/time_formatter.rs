use bson::Bson;
use chrono::Utc;
pub fn get_current_time_bson() -> Bson {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    Bson::String(formatted_time)
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_get_current_time_bson_returns_string_type() {
        let result = get_current_time_bson();
        assert!(matches!(result, Bson::String(_)));
    }

    #[test]
    fn test_get_current_time_bson_format() {
        let result = get_current_time_bson();
        if let Bson::String(timestamp) = result {
            let parsed = DateTime::parse_from_rfc3339(&timestamp);
            assert!(parsed.is_ok(), "Timestamp should be valid RFC3339 format");
        } else {
            panic!("Expected Bson::String variant");
        }
    }

    #[test]
    fn test_get_current_time_bson_milliseconds() {
        let result = get_current_time_bson();
        if let Bson::String(timestamp) = result {
            assert!(timestamp.contains('.'), "Timestamp should include decimal point for milliseconds");
            let parts: Vec<&str> = timestamp.split('.').collect();
            assert_eq!(parts.len(), 2, "Timestamp should have millisecond portion");
            let milliseconds = parts[1].chars().take_while(|c| c.is_numeric()).count();
            assert_eq!(milliseconds, 3, "Should have exactly 3 millisecond digits");
        } else {
            panic!("Expected Bson::String variant");
        }
    }
}
// Helper function to parse ReplayGain values
pub fn parse_replaygain_value(value: &str) -> Option<f64> {
    // ReplayGain values are typically in format: "+1.23 dB" or "-1.23 dB"
    value
        .trim()
        .trim_end_matches("dB")
        .trim_end_matches("db")
        .trim()
        .parse::<f64>()
        .ok()
}

// Helper function to format ReplayGain gain value
pub fn format_replaygain_gain(value: f64) -> String {
    format!("{:+.2} dB", value)
}

// Helper function to format ReplayGain peak value
pub fn format_replaygain_peak(value: f64) -> String {
    format!("{:.6}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_replaygain_value_valid_formats() {
        // Test various valid formats
        assert_eq!(parse_replaygain_value("+1.23 dB"), Some(1.23));
        assert_eq!(parse_replaygain_value("-1.23 dB"), Some(-1.23));
        assert_eq!(parse_replaygain_value("+0.00 dB"), Some(0.0));
        assert_eq!(parse_replaygain_value("-5.67 dB"), Some(-5.67));
        assert_eq!(parse_replaygain_value("+12.34 dB"), Some(12.34));

        // Test with different case and whitespace
        assert_eq!(parse_replaygain_value("+1.23 db"), Some(1.23));
        assert_eq!(parse_replaygain_value("  +1.23 dB  "), Some(1.23));
        assert_eq!(parse_replaygain_value("+1.23dB"), Some(1.23));
        assert_eq!(parse_replaygain_value("+1.23db"), Some(1.23));
    }

    #[test]
    fn test_parse_replaygain_value_invalid_formats() {
        // Test invalid formats that should return None
        assert_eq!(parse_replaygain_value(""), None);
        assert_eq!(parse_replaygain_value("invalid"), None);
        assert_eq!(parse_replaygain_value("dB"), None);
        assert_eq!(parse_replaygain_value("+1.23"), Some(1.23)); // dB suffix is optional
        assert_eq!(parse_replaygain_value("abc dB"), None);
    }

    #[test]
    fn test_format_replaygain_gain() {
        // Test positive values
        assert_eq!(format_replaygain_gain(1.23), "+1.23 dB");
        assert_eq!(format_replaygain_gain(12.34), "+12.34 dB");

        // Test negative values
        assert_eq!(format_replaygain_gain(-1.23), "-1.23 dB");
        assert_eq!(format_replaygain_gain(-5.67), "-5.67 dB");

        // Test zero
        assert_eq!(format_replaygain_gain(0.0), "+0.00 dB");

        // Test rounding
        assert_eq!(format_replaygain_gain(1.23456), "+1.23 dB");
        assert_eq!(format_replaygain_gain(-1.23456), "-1.23 dB");
    }

    #[test]
    fn test_format_replaygain_peak() {
        // Test various peak values
        assert_eq!(format_replaygain_peak(0.123456), "0.123456");
        assert_eq!(format_replaygain_peak(1.0), "1.000000");
        assert_eq!(format_replaygain_peak(0.999999), "0.999999");
        assert_eq!(format_replaygain_peak(0.0), "0.000000");
        assert_eq!(format_replaygain_peak(0.123456789), "0.123457"); // Rounding test

        // Test negative values (though peak should typically be positive)
        assert_eq!(format_replaygain_peak(-0.123456), "-0.123456");
    }

    #[test]
    fn test_parse_and_format_round_trip() {
        // Test that parsing and formatting work together
        let test_values = [1.23, -1.23, 0.0, 12.34, -5.67];

        for &value in &test_values {
            let formatted = format_replaygain_gain(value);
            let parsed = parse_replaygain_value(&formatted);
            assert_eq!(
                parsed,
                Some(value),
                "Round trip failed for value: {}",
                value
            );
        }
    }
}

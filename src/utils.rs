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

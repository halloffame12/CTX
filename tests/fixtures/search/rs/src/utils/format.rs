pub const FORMAT_VERSION: &str = "2";

pub fn format_date(d: u64) -> String {
    d.to_string()
}

pub fn format_currency(n: f64) -> String {
    format!("${n:.2}")
}

fn internal_helper(x: u64) -> u64 {
    x + 1
}
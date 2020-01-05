use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn rand_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect()
}

pub fn format_seconds(secs: i64) -> String {
    let weeks = secs / 604800;
    let days = (secs % 604800) / 86400;
    let hours = ((secs % 604800) % 86400) / 3600;
    let minutes = (((secs % 604800) % 86400) % 3600) / 60;
    let seconds = (((secs % 604800) % 86400) % 3600) % 60;

    format!(
        "{}w, {}d, {}h, {}m, {}s",
        weeks, days, hours, minutes, seconds
    )
}

pub fn parse_tags(tags: Option<&str>) -> Option<Vec<String>> {
    match tags {
        Some(tags_string) => Some(
            tags_string
                .split(",")
                .map(|t| t.into())
                .collect()),
        None => None
    }
}

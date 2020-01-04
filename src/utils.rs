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

pub fn serialize_ints(v: Option<Vec<i32>>) -> Option<String> {
    match v {
        Some(values) => {
            let result = values
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",");

            Some(result)
        },
        None => None
    }
}

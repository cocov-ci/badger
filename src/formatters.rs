pub(crate) fn format_coverage(cov: String) -> (String, String) {
    let parsed_coverage: i8 = cov
        .parse()
        .unwrap_or(-1);

    let color = match parsed_coverage {
        -1 => "lightgrey",
        90..=100 => "brightgreen",
        70..=89 => "green",
        50..=69 => "yellow",
        _ => "red",
    };
    let label = if parsed_coverage == -1 {
        "unknown".into()
    } else {
        format!("{}%", parsed_coverage)
    };
    (color.into(), label)
}

pub(crate) fn format_issues(num: String) -> (String, String) {
    let parsed: i8 = num
        .parse()
        .unwrap_or(-1);

    let color = match parsed {
        -1 => "lightgrey",
        0 => "brightgreen",
        _ => "important",
    };

    (color.into(), num)
}

use crate::consts::DEFAULT_VERSION;

const VERSION_INI: &str = include_str!("../version.ini");

pub(crate) fn load() -> String {
    parse(VERSION_INI).unwrap_or_else(|| DEFAULT_VERSION.to_string())
}

fn parse(text: &str) -> Option<String> {
    for raw in text.lines() {
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty()
            || line.starts_with('[')
            || line.starts_with(';')
            || line.starts_with('#')
        {
            continue;
        }
        let value = match line.split_once('=') {
            Some((key, value)) if key.trim().eq_ignore_ascii_case("version") => value.trim(),
            Some(_) => continue,
            None => line,
        };
        let value = value.trim().trim_matches('"').trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

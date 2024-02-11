use cyu_fetcher::calendar::GetCalendarResponseElement;
use once_cell::sync::Lazy;
use regex::Regex;

pub type Event = GetCalendarResponseElement;

pub fn parse_description(description: &String) -> String {
    static LINEBREAKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\r\n|<br />)+").unwrap());
    html_escape::decode_html_entities(LINEBREAKS_REGEX.replace_all(description, "\n").trim())
        .to_string()
}

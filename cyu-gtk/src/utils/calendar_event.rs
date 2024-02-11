use cyu_fetcher::calendar::GetCalendarResponseElement;
use once_cell::sync::Lazy;
use regex::Regex;

pub type Event = GetCalendarResponseElement;

pub static LINEBREAKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\r\n|<br />)+").unwrap());

pub fn parse_description(description: &String) -> String {
    html_escape::decode_html_entities(LINEBREAKS_REGEX.replace_all(description, "\n").trim())
        .to_string()
}

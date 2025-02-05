use super::Auth;
use anyhow::{Context as _, Result};
use cyu_fetcher::calendar::ColorBy;
use cyu_fetcher::Fetcher;
use icalendar::{Component as _, EventLike as _};
use std::borrow::Cow;

pub async fn generate(fetcher: &Fetcher, auth: Auth) -> Result<String> {
    let events = fetcher
        .get_all_calendar(cyu_fetcher::calendar::GetAllQuery {
            id: auth.id,
            token: auth.token,
            color_by: ColorBy::EventCategory,
        })
        .await
        // .map_err(|_| Error::RemoteError)?;
        .context("Failed to get all calendar")?;

    let events = events.into_iter().filter_map(|event| {
        let mut ievent = icalendar::Event::new();

        let description = event.description();

        ievent
            .uid(&format!("{}@cyu-calendar", event.id()))
            .description(&description);

        match (event.all_day(), event.end()) {
            (true, _) => ievent.all_day(event.start().as_utc().date_naive()),
            (false, Some(end)) => ievent.starts(event.start().as_utc()).ends(end.as_utc()),
            (false, None) => return None,
        };

        let category = event.event_category();
        let title: Cow<str> = match category.as_str() {
            // "CM" => format!("CM {}", event.description().split('\n').rev().nth(2).unwrap_or_default()).into(),
            // "TD" => description.split('\n').rev().nth(2).unwrap_or_default().into(),
            "CM" | "TD" => format!(
                "{} {}",
                category,
                description
                    .split('\n')
                    .rev()
                    .nth(2)
                    .unwrap_or_default()
                    .replace(category, "")
            )
            .into(),
            cat => cat.into(),
        };
        ievent.summary(&title);

        Some(ievent.done())
    });

    let mut calendar = icalendar::Calendar::from_iter(events);
    let calendar = calendar.name("CYU Calendar");

    Ok(calendar.to_string().replace("\\N", "\\n"))
}

pub mod auth;
pub mod calendar;
pub mod errors;
pub mod utils;

pub use errors::Error;

#[derive(Clone)]
pub struct Fetcher {
    pub requester: reqwest::Client,
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            requester: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        }
    }

    pub async fn login(&self, username: String, password: String) -> Result<String, Error> {
        auth::login(&self.requester, username, password).await
    }

    pub async fn get_infos(&self, token: String) -> Result<auth::InfosResponse, Error> {
        auth::get_infos(&self.requester, token).await
    }

    pub async fn get_calendar(
        &self,
        query: calendar::GetCalendarQuery,
    ) -> Result<calendar::GetCalendarResponse, Error> {
        calendar::get_calendar(&self.requester, query).await
    }

    pub async fn get_calendar_limits(
        &self,
        query: calendar::GetLimitsQuery<'_>
    ) -> Result<calendar::GetLimitsResponse, Error> {
        calendar::get_limits(&self.requester, query).await
    }

    pub async fn get_all_calendar(&self, query: calendar::GetAllQuery) -> Result<calendar::GetCalendarResponse, Error> {
        calendar::get_all(&self.requester, query).await
    }
}

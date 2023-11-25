pub mod auth;
pub mod calendar;

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

    pub async fn login(&self, username: String, password: String) -> Result<String, ()> {
        auth::login(&self.requester, username, password).await
    }

    pub async fn get_infos(&self, token: String) -> Result<auth::InfosResponse, ()> {
        auth::get_infos(&self.requester, token).await
    }

    pub async fn get_calendar(
        &self,
        query: calendar::GetCalendarQuery,
    ) -> Result<calendar::GetCalendarResponse, ()> {
        calendar::get_calendar(&self.requester, query).await
    }
}

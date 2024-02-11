use cyu_fetcher::Fetcher;
use once_cell::sync::Lazy;

pub const FETCHER: Lazy<Fetcher> = Lazy::new(Fetcher::new);

use axum::extract::FromRef;
use cyu_fetcher::Fetcher;
use handlebars::Handlebars;
use std::{path::PathBuf, sync::Arc};

pub type TemplateEngine = Arc<Handlebars<'static>>;

#[derive(Clone)]
pub struct App {
    requester: Fetcher,
    template_engine: TemplateEngine,
}

impl App {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        let hbs_path = PathBuf::from(env!("CARGO_PKG_NAME")).join("views");
        handlebars
            .register_template_file("layout", hbs_path.join("layout.hbs"))
            .expect("Failed to register layout template");
        handlebars
            .register_templates_directory(hbs_path.join("pages"), Default::default())
            .expect("Failed to register pages templates");
        handlebars.set_dev_mode(true);
        Self {
            requester: Fetcher::new(),
            template_engine: handlebars.into(),
        }
    }
}

impl FromRef<App> for Fetcher {
    fn from_ref(app: &App) -> Self {
        app.requester.clone()
    }
}

impl FromRef<App> for TemplateEngine {
    fn from_ref(app: &App) -> Self {
        app.template_engine.clone()
    }
}

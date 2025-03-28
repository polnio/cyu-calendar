use crate::utils::Env;
use anyhow::{Context as _, Result};
use axum::extract::FromRef;
use base64::Engine;
use cyu_fetcher::Fetcher;
use handlebars::Handlebars;
use rust_embed::Embed;
use std::sync::Arc;

pub type TemplateEngine = Arc<Handlebars<'static>>;
pub type Encrypter = Arc<auth_token::Encrypter>;
pub type Database = Arc<sqlx::SqlitePool>;

#[derive(Embed)]
#[folder = "assets/views"]
struct Views;

fn to_json(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).unwrap();

    let json = serde_json::to_string(param.value()).unwrap();
    // out.write(param.value().render().as_ref())?;
    out.write(&json)?;
    Ok(())
}

#[derive(Clone)]
pub struct App {
    requester: Fetcher,
    template_engine: TemplateEngine,
    // env: Env,
    encrypter: Encrypter,
    database: Database,
}

impl App {
    pub async fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        for path in Views::iter() {
            if !path.ends_with(".hbs") {
                continue;
            }
            let name = path.split('/').last().unwrap();
            let name = &name[..name.len() - 4];
            let content = Views::get(&path).unwrap();
            let content = std::str::from_utf8(&content.data).context("Failed to parse template")?;
            handlebars
                .register_template_string(name, content)
                .with_context(|| format!("Failed to register template '{}'", name))?;
        }
        handlebars.register_helper("json", Box::new(to_json));
        handlebars.set_dev_mode(true);
        let env = Env::load().context("Failed to load environment variables")?;
        let key = base64::engine::general_purpose::STANDARD
            .decode(env.ics_auth_key)
            .unwrap();
        let encrypter = auth_token::Encrypter::new(&key).unwrap();
        let database = sqlx::SqlitePool::connect(&env.database_url).await.unwrap();
        Ok(Self {
            requester: Fetcher::new(),
            template_engine: handlebars.into(),
            // env: Env::load(),
            encrypter: encrypter.into(),
            database: database.into(),
        })
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

impl FromRef<App> for Encrypter {
    fn from_ref(app: &App) -> Self {
        app.encrypter.clone()
    }
}

impl FromRef<App> for Database {
    fn from_ref(app: &App) -> Self {
        app.database.clone()
    }
}

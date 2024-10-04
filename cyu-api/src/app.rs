use axum::extract::FromRef;
use cyu_fetcher::Fetcher;
use handlebars::Handlebars;
use std::{path::PathBuf, sync::Arc};

pub type TemplateEngine = Arc<Handlebars<'static>>;

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
}

impl App {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        let hbs_path = PathBuf::from(env!("CARGO_PKG_NAME"))
            .join("assets")
            .join("views");

        let _ = handlebars
            .register_template_file("layout", hbs_path.join("layout.hbs"))
            .map_err(|err| {
                eprintln!("Failed to register layout template: {}", err);
                std::process::exit(1);
            });
        let _ = handlebars
            .register_templates_directory(hbs_path.join("pages"), Default::default())
            .map_err(|err| {
                eprintln!("Failed to register pages templates: {}", err);
                std::process::exit(1);
            });
        handlebars.register_helper("json", Box::new(to_json));
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

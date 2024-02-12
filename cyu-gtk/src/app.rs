use crate::pages::{
    calendar::{CalendarPage, CalendarPageInput, CalendarPageOutput},
    login::{LoginPage, LoginPageOutput},
};
use crate::utils::auth::{self, AUTH};
use crate::utils::constants::APP_TITLE;
use relm4::{adw::prelude::*, prelude::*};
use relm4::{loading_widgets::LoadingWidgets, view, AsyncComponentSender};

pub struct App {
    login_page: Controller<LoginPage>,
    calendar_page: AsyncController<CalendarPage>,

    is_authed: bool,
}

impl App {
    fn current_page(&self) -> &dyn AsRef<gtk::Widget> {
        if self.is_authed {
            self.calendar_page.widget()
        } else {
            self.login_page.widget()
        }
    }
}

#[derive(Debug)]
pub enum AppInput {
    LoggedIn,
    LoggedOut,
}
impl From<LoginPageOutput> for AppInput {
    fn from(msg: LoginPageOutput) -> Self {
        match msg {
            LoginPageOutput::LoggedIn => Self::LoggedIn,
        }
    }
}
impl From<CalendarPageOutput> for AppInput {
    fn from(msg: CalendarPageOutput) -> Self {
        match msg {
            CalendarPageOutput::LoggedOut => Self::LoggedOut,
        }
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for App {
    type Init = ();
    type Input = AppInput;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            gtk::Stack {
                add_child: calendar_page,
                add_child: login_page,
                #[watch]
                set_visible_child: model.current_page().as_ref(),
            }
        },
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_title: Some(APP_TITLE),
                set_default_size: (800, 600),

                #[name = "spinner"]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                },
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        auth::init().await;

        let model = Self {
            login_page: LoginPage::builder()
                .launch(())
                .forward(sender.input_sender(), AppInput::from),
            calendar_page: CalendarPage::builder()
                .launch(())
                .forward(sender.input_sender(), AppInput::from),
            is_authed: { AUTH.read().unwrap().is_some() },
        };

        let login_page = model.login_page.widget();
        let calendar_page = model.calendar_page.widget();
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<App>) {
        match msg {
            AppInput::LoggedIn => {
                self.is_authed = true;
                self.calendar_page.emit(CalendarPageInput::Refresh);
            }
            AppInput::LoggedOut => {
                self.is_authed = false;
            }
        }
    }
}

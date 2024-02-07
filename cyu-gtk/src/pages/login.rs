use crate::utils::auth;
use relm4::{adw::prelude::*, prelude::*};

pub struct LoginPage {
    username_entry: adw::EntryRow,
    password_entry: adw::PasswordEntryRow,
    submit_button: gtk::Button,
}

#[derive(Debug)]
pub enum LoginPageInput {
    Submit,
}

#[derive(Debug)]
pub enum LoginPageOutput {
    LoggedIn,
}

#[relm4::component(pub)]
impl SimpleComponent for LoginPage {
    type Init = ();
    type Input = LoginPageInput;
    type Output = LoginPageOutput;

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {},

            #[wrap(Some)]
            set_content = &adw::Clamp {
                gtk::ListBox {
                    set_valign: gtk::Align::Center,
                    #[name = "username_entry"]
                    adw::EntryRow {
                        set_title: "Nom d'utilisateur",
                        connect_activate => LoginPageInput::Submit,
                    },
                    #[name = "password_entry"]
                    adw::PasswordEntryRow {
                        set_title: "Mot de passe",
                        connect_activate => LoginPageInput::Submit,
                    },
                    #[name = "submit_button"]
                    gtk::Button {
                        set_label: "Se connecter",
                        add_css_class: "suggested-action",
                        connect_clicked => LoginPageInput::Submit,
                    },
                },
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let model = Self {
            username_entry: widgets.username_entry.clone(),
            password_entry: widgets.password_entry.clone(),
            submit_button: widgets.submit_button.clone(),
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            LoginPageInput::Submit => {
                let submit_button = self.submit_button.clone();

                let username = self.username_entry.text();
                let password = self.password_entry.text();

                gtk::glib::spawn_future_local(async move {
                    submit_button.set_sensitive(false);
                    let auth_result = auth::login(username.into(), password.into()).await;
                    match auth_result {
                        Ok(_) => {
                            sender
                                .output(LoginPageOutput::LoggedIn)
                                .expect("failed to send login signal");
                        }
                        Err(auth::Error::BadCredentials) => {
                            println!("Bad credentials");
                        }
                        _ => {
                            println!("An error occurred");
                        }
                    }
                    submit_button.set_sensitive(true);
                });
            }
        }
    }
}

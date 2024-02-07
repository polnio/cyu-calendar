use crate::utils::calendar_event::{parse_description, Event};
use relm4::{adw::prelude::*, prelude::*};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

pub struct CalendarEventDetailsWidget {
    event: Option<Event>,
    stack_widget: gtk::Stack,
    event_widget: adw::Clamp,
    no_event_widget: adw::StatusPage,
}
impl CalendarEventDetailsWidget {}

#[derive(Debug)]
pub enum CalendarEventDetailsWidgetInput {
    SetEvent(Option<Event>),
}

#[relm4::component(pub)]
impl SimpleComponent for CalendarEventDetailsWidget {
    type Init = Option<Event>;
    type Input = CalendarEventDetailsWidgetInput;
    type Output = ();

    view! {
        root = gtk::Stack {
            #[local_ref]
            no_event_widget -> adw::StatusPage {
                set_icon_name: Some("list-view"),
                set_title: "Pas d'évènement sélectionné",
                set_description: Some("Selectionnez un évènement"),
            },
            #[local_ref]
            event_widget -> adw::Clamp {
                gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                    adw::ActionRow {
                        set_title: "Description",
                        #[watch]
                        set_subtitle: &model.event.as_ref().map(|event| parse_description(event.description())).unwrap_or_default(),
                        add_css_class: "property",
                    },
                    adw::ActionRow {
                        set_title: "Heure",
                        #[watch]
                        set_subtitle: &model.event.as_ref().map(|event| {
                            if *event.all_day() {
                                "Toute la journée".to_string()
                            } else if let Some(end) = event.end() {
                                format!(
                                    "{} à {}",
                                    event.start().format("%H:%M"),
                                    end.format("%H:%M")
                                )
                            } else {
                                event.start().format("%H:%M").to_string()
                            }
                        }).unwrap_or_default(),
                        add_css_class: "property",
                    },
                    adw::ActionRow {
                        set_title: "Département",
                        #[watch]
                        set_subtitle: model.event.as_ref().map(|event| event.department()).unwrap_or(&String::from("")),
                        add_css_class: "property",
                    },
                    adw::ActionRow {
                        set_title: "Faculté",
                        #[watch]
                        set_visible: model.event.as_ref().and_then(|event| event.faculty().as_ref()).is_some(),
                        #[watch]
                        set_subtitle: model.event.as_ref().and_then(|event| event.faculty().as_ref()).unwrap_or(&String::from("")),
                        add_css_class: "property",
                    },
                }
            }
        }
    }

    fn init(
        event: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let event_widget = adw::Clamp::default();
        let no_event_widget = adw::StatusPage::default();
        let model = Self {
            event,
            stack_widget: root.clone(),
            event_widget: event_widget.clone(),
            no_event_widget: no_event_widget.clone(),
        };
        let widgets = view_output!();
        widgets.root.set_visible_child(&model.no_event_widget);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            CalendarEventDetailsWidgetInput::SetEvent(event) => {
                if event.is_some() {
                    self.stack_widget.set_visible_child(&self.event_widget);
                } else {
                    self.stack_widget.set_visible_child(&self.no_event_widget);
                }
                self.event = event;
            }
        }
    }
}

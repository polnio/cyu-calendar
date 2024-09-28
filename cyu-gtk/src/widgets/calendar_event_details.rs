use crate::utils::calendar_event::{parse_description, Event};
use crate::widgets::calendar_event_map::CalendarEventMap;
use relm4::{adw::prelude::*, prelude::*};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

use super::calendar_event_map::CalendarEventMapInput;

pub struct CalendarEventDetailsWidget {
    event: Option<Event>,
    stack_widget: gtk::Stack,
    event_widget: adw::Clamp,
    no_event_widget: adw::StatusPage,
    map_widget: Controller<CalendarEventMap>,
}

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
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::ListBox {
                        set_valign: gtk::Align::Start,
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::ActionRow {
                            set_title: "Description",
                            #[watch]
                            set_subtitle: &model.event
                                .as_ref()
                                .map(|event| event.description())
                                .unwrap_or_default(),
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
                            set_title: "Site",
                            #[watch]
                            set_subtitle: model.event
                                .as_ref()
                                .and_then(|event| event.sites().as_ref())
                                .and_then(|sites| sites.first())
                                .unwrap_or(&String::default()),
                            add_css_class: "property",
                        },
                        adw::ActionRow {
                            set_title: "Faculté",
                            #[watch]
                            set_visible: model.event
                                .as_ref()
                                .and_then(|event| event.faculty().as_ref())
                                .is_some(),
                            #[watch]
                            set_subtitle: model.event
                                .as_ref()
                                .and_then(|event| event.faculty().as_ref())
                                .unwrap_or(&String::default()),
                            add_css_class: "property",
                        },
                    },
                    append: map_widget
                }
            }
        }
    }

    fn init(
        event: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            event,
            stack_widget: root.clone(),
            event_widget: adw::Clamp::default(),
            no_event_widget: adw::StatusPage::default(),
            map_widget: CalendarEventMap::builder().launch(None).detach(),
        };

        let event_widget = model.event_widget.clone();
        let no_event_widget = model.no_event_widget.clone();
        let map_widget = model.map_widget.widget();
        let widgets = view_output!();
        widgets.root.set_visible_child(&model.no_event_widget);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            CalendarEventDetailsWidgetInput::SetEvent(event) => {
                if event.is_some() {
                    self.map_widget
                        .sender()
                        .send(CalendarEventMapInput::SetEvent(event.clone()))
                        .expect("failed to send event details widget");
                    self.stack_widget.set_visible_child(&self.event_widget);
                } else {
                    self.stack_widget.set_visible_child(&self.no_event_widget);
                }
                self.event = event;
            }
        }
    }
}

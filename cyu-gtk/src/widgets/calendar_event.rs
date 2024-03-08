use crate::utils::calendar_event::{parse_description, Event};
use relm4::{factory::FactoryComponent, gtk::prelude::*, prelude::*, FactorySender, RelmWidgetExt};

pub struct CalendarEventWidget {
    event: Event,
}

#[derive(Debug)]
pub enum CalendarEventWidgetInput {
    Clicked,
}

#[derive(Debug)]
pub enum CalendarEventWidgetOutput {
    Clicked(Event),
}

#[relm4::factory(pub)]
impl FactoryComponent for CalendarEventWidget {
    type Init = Event;
    type Input = CalendarEventWidgetInput;
    type Output = CalendarEventWidgetOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        root = gtk::Box {
            add_css_class: "card",
            set_orientation: gtk::Orientation::Vertical,
            inline_css: &format!("border-left: 3px solid {}; padding-left: 10px;", self.event.background_color()),
            gtk::Label {
                set_halign: gtk::Align::Start,
                set_label: {
                    let start = self.event.start().format("%H:%M");
                    let end = self.event.end().map(|end| end.format("%H:%M").to_string()).unwrap_or("".to_string());
                    &format!("{} - {}", start, end)
                },
            },
            gtk::Label {
                set_halign: gtk::Align::Start,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
                set_label: &parse_description(self.event.description()),
            },
            add_controller = gtk::GestureClick {
                connect_released[sender] => move |gesture, _, _, _| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    sender.input(CalendarEventWidgetInput::Clicked)
                },
            },
        },
    }

    fn init_model(event: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self { event }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            CalendarEventWidgetInput::Clicked => {
                sender
                    .output(CalendarEventWidgetOutput::Clicked(self.event.clone()))
                    .expect("failed to send click signal");
            }
        }
    }
}

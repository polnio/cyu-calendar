use crate::utils::auth::{self, AUTH};
use crate::utils::calendar_event::Event;
use crate::utils::constants::APP_TITLE;
use crate::utils::FETCHER;
use crate::widgets::calendar_event::{CalendarEventWidget, CalendarEventWidgetOutput};
use crate::widgets::calendar_event_details::{
    CalendarEventDetailsWidget, CalendarEventDetailsWidgetInput,
};
use adw::BreakpointCondition;
use cyu_fetcher::calendar::{CalendarView, ColorBy, GetCalendarQuery};
use cyu_fetcher::errors::Error;
use relm4::factory::FactoryVecDeque;
use relm4::{adw::prelude::*, prelude::*};

pub struct CalendarPage {
    event_widgets: FactoryVecDeque<CalendarEventWidget>,
    day_selector_widget: gtk::Calendar,
    event_details_widget: Controller<CalendarEventDetailsWidget>,
    split_view: adw::NavigationSplitView,
}

impl CalendarPage {
    async fn refresh(&mut self, sender: &AsyncComponentSender<Self>) {
        let auth = AUTH.read().unwrap();
        let Some(auth) = auth.as_ref() else {
            return;
        };

        let date = self
            .day_selector_widget
            .date()
            .format("%Y-%m-%d")
            .expect("Could not format date")
            .to_string();
        let calendar = FETCHER
            .get_calendar(GetCalendarQuery {
                id: auth.id.clone(),
                token: auth.token.clone(),
                start: date.clone(),
                end: date,
                view: CalendarView::Day,
                color_by: ColorBy::Subject,
            })
            .await;

        match calendar {
            Ok(mut calendar) => {
                let event_widgets = &mut self.event_widgets.guard();
                calendar.sort_by(|a, b| a.start().cmp(b.start()));
                event_widgets.clear();
                for event in &calendar {
                    event_widgets.push_back(event.clone());
                }
            }
            Err(Error::Unauthorized) => {
                auth::logout().await;
                sender
                    .output(CalendarPageOutput::LoggedOut)
                    .expect("Failed to send logout signal");
            }
            Err(err) => {
                println!("Failed to get calendar: {:?}", err);
            }
        }
    }
}

#[derive(Debug)]
pub enum CalendarPageInput {
    LogOut,
    Refresh,
    OpenDetails(Event),
}
impl From<CalendarEventWidgetOutput> for CalendarPageInput {
    fn from(output: CalendarEventWidgetOutput) -> Self {
        match output {
            CalendarEventWidgetOutput::Clicked(event) => CalendarPageInput::OpenDetails(event),
        }
    }
}

#[derive(Debug)]
pub enum CalendarPageOutput {
    LoggedOut,
}

#[relm4::component(async, pub)]
impl AsyncComponent for CalendarPage {
    type Init = ();
    type Input = CalendarPageInput;
    type Output = CalendarPageOutput;
    type CommandOutput = ();

    view! {
        adw::BreakpointBin {
            #[wrap(Some)]
            #[local_ref]
            set_child = &split_view -> adw::NavigationSplitView {
                #[wrap(Some)]
                set_sidebar = &adw::NavigationPage {
                    set_title: APP_TITLE,
                    adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            pack_start = &gtk::Button {
                                set_label: "Logout",
                                connect_clicked => CalendarPageInput::LogOut,
                            }
                        },
                        #[wrap(Some)]
                        set_content = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            #[local_ref]
                            day_selector_widget -> gtk::Calendar {
                                // connect_day_selected => CalendarPageInput::DaySelected,
                                connect_day_selected => CalendarPageInput::Refresh,
                            },
                            gtk::ScrolledWindow {
                                set_vexpand: true,
                                #[local_ref]
                                event_widgets -> gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                }
                            }
                        }
                    },
                },
                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    set_title: "Content",
                    adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            set_show_title: false,
                        },
                        set_content: Some(model.event_details_widget.widget()),
                    },
                },
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let event_widgets = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), CalendarPageInput::from);
        let event_details_widget = CalendarEventDetailsWidget::builder().launch(None).detach();
        let day_selector_widget = gtk::Calendar::new();
        let split_view = adw::NavigationSplitView::default();

        let mut model = Self {
            event_widgets,
            event_details_widget,
            day_selector_widget: day_selector_widget.clone(),
            split_view: split_view.clone(),
        };
        model.refresh(&sender).await;

        let event_widgets = model.event_widgets.widget();
        let widgets = view_output!();

        let breakpoint =
            adw::Breakpoint::new(BreakpointCondition::parse("max-width: 700sp").unwrap());
        breakpoint.add_setter(&widgets.split_view, "collapsed", &true.to_value());
        root.add_breakpoint(breakpoint);

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            CalendarPageInput::LogOut => {
                gtk::glib::spawn_future_local(async move {
                    auth::logout().await;
                    sender
                        .output(CalendarPageOutput::LoggedOut)
                        .expect("failed to send logout signal");
                });
            }
            CalendarPageInput::Refresh => {
                self.refresh(&sender).await;
            }
            CalendarPageInput::OpenDetails(event) => {
                self.event_details_widget
                    .sender()
                    .send(CalendarEventDetailsWidgetInput::SetEvent(Some(event)))
                    .expect("failed to send event details widget");
                self.split_view.set_show_content(true);
            }
        }
    }
}

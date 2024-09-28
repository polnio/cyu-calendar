use crate::utils::calendar_event::Event;
use cyu_fetcher::calendar::{LOCATIONS_COORD, NB_LOCATIONS};
use libshumate::prelude::*;
use relm4::{gtk, prelude::*, SimpleComponent};

pub struct CalendarEventMap {
    event: Option<Event>,
    map_widget: libshumate::SimpleMap,
}
impl CalendarEventMap {
    fn update(&self) {
        let Some(coords) = self.event.as_ref().and_then(|event| event.coords()) else {
            return;
        };
        let Some(viewport) = self.map_widget.viewport() else {
            return;
        };
        viewport.set_location(coords[0], coords[1]);
        viewport.set_zoom_level(30.0);
    }

    pub fn set_event(&mut self, event: Option<Event>) {
        self.event = event;
        CalendarEventMap::update(self);
    }
}

#[derive(Debug)]
pub enum CalendarEventMapInput {
    SetEvent(Option<Event>),
}

#[relm4::component(pub)]
impl SimpleComponent for CalendarEventMap {
    type Init = Option<Event>;
    type Input = CalendarEventMapInput;
    type Output = ();

    view! {
        libshumate::SimpleMap {
            set_vexpand: true,
            set_margin_vertical: 20,
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: relm4::prelude::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            event: init,
            map_widget: root.clone(),
        };
        let widgets = view_output!();
        let map_source =
            libshumate::MapSourceRegistry::with_defaults().by_id(libshumate::MAP_SOURCE_OSM_MAPNIK);
        model.map_widget.set_map_source(map_source.as_ref());
        if let Some(viewport) = model.map_widget.viewport() {
            let path_layer = libshumate::PathLayer::new(&viewport);
            let marker_layer = libshumate::MarkerLayer::new(&viewport);
            for i in 0..NB_LOCATIONS {
                let icon = gtk::Image::from_resource(
                    "/fr/poco/cyu-gtk/icons/private/hicolor/scalable/apps/maps-mark-location.svg",
                );
                icon.set_pixel_size(35);
                let marker = libshumate::Marker::new();
                marker.set_location(LOCATIONS_COORD[i][0], LOCATIONS_COORD[i][1]);
                marker.set_child(Some(&icon));
                marker_layer.add_marker(&marker);
                path_layer.add_node(&marker);
            }
            model.map_widget.add_overlay_layer(&marker_layer);
        }
        model.update();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            CalendarEventMapInput::SetEvent(event) => {
                self.set_event(event);
            }
        }
    }
}

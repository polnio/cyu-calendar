mod app;
mod pages;
mod utils;
mod widgets;

use relm4::{gtk::gio, RelmApp};
use utils::constants::APP_ID;

fn main() {
    gio::resources_register_include!("cyu-gtk.gresource").expect("Failed to register resources");
    RelmApp::new(APP_ID).run_async::<app::App>(());
}

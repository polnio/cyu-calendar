mod app;
mod pages;
mod utils;
mod widgets;

use relm4::RelmApp;
use utils::constants::APP_ID;

fn main() {
    RelmApp::new(APP_ID).run_async::<app::App>(());
}

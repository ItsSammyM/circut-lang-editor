#![allow(unused, clippy::all)]

mod editor;

use editor::App;

fn main() -> Result<(), eframe::Error> {
    App::run()
}

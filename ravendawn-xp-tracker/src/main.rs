#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;

mod external;
mod gui;

// Games
mod ravendawn;

lazy_static! {
    pub static ref WINDOW_WIDTH: f32 = 350.0;
    pub static ref WINDOW_HEIGHT: f32 = 180.0;
}

fn main() {
    gui::draw_window();
}

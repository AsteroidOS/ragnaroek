//! This crate provides an FLTK-based graphical frontend for ragnaroek.

use fltk::{app::App, button::Button, prelude::*, window::Window};
use fltk_theme::{widget_themes, ThemeType, WidgetTheme};

fn main() {
    let app = App::default();
    // Choose a theme to match system dark mode state
    // TODO: Re-check and re-theme regularly rather than only at app start
    let widget_theme = match dark_light::detect() {
        dark_light::Mode::Light => WidgetTheme::new(ThemeType::Classic),
        dark_light::Mode::Dark => WidgetTheme::new(ThemeType::Dark),
    };
    widget_theme.apply();

    let mut win = Window::default().with_size(400, 300);
    let mut btn = Button::new(160, 200, 80, 30, "Print PIT");
    btn.set_frame(widget_themes::OS_DEFAULT_BUTTON_UP_BOX);
    btn.set_callback(move |_| on_print_pit());

    // Create a box at the bottom of the screen showing log output

    win.end();
    win.show();
    app.run().unwrap();
}

fn on_print_pit() {
    println!("Pressed!");
}

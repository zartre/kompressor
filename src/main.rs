#![windows_subsystem = "windows"]

mod app;
mod compression;
mod format;
mod image_item;
mod theme;

use gpui::{
    actions, px, size, App, AppContext as _, Application, Bounds, KeyBinding, Menu, MenuItem,
    TitlebarOptions, WindowBounds, WindowOptions,
};

use app::Kompressor;

actions!(Kompressor, [Close, Minimize, Quit]);

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(450.), px(225.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Kompressor".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Kompressor::new(window, cx)),
        )
        .unwrap();

        register_shortcuts(cx);

        cx.activate(true);

        cx.set_menus(vec![Menu {
            name: "Kompressor".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);
    });
}

fn register_shortcuts(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-w", Close, None),
        KeyBinding::new("cmd-m", Minimize, None),
        KeyBinding::new("cmd-q", Quit, None),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
}

// Comment for enabling console
#![windows_subsystem = "windows"]

use config::launch::AppLaunchConfig;
use eframe::egui::*;
use std::env;

mod app;
mod backend;
mod config;
mod data;
mod logger;
mod paths;
mod theme;
mod views;

fn main() {
    logger::init();

    backend::init();

    let args: Vec<String> = env::args().collect();

    let init_config = AppLaunchConfig::from(&args).unwrap_or_default();
    let gui = app::BepInExGUI::new(init_config.clone());

    let native_options = eframe::NativeOptions {
        min_window_size: Some(Vec2::new(240., 270.)),
        initial_window_size: Some(Vec2::new(1034., 520.)),
        window_builder: Some(Box::new(move |builder| {
            builder.with_title(init_config.window_title())
        })),

        icon_data: Some(load_icon()),

        ..Default::default()
    };

    match eframe::run_native(
        app::NAME,
        native_options,
        Box::new(|cc| Box::new(gui.init(cc))),
    ) {
        Ok(_) => {}
        Err(res) => tracing::error!("{:?}", res),
    }
}

fn load_icon() -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../assets/icons/ror2_discord_server_icon.png");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

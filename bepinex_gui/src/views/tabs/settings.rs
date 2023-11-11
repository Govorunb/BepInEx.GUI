use std::sync::atomic::Ordering;

use eframe::{egui::{CentralPanel, Context, ScrollArea, Slider, TextStyle, Ui}, epaint::Vec2};

use crate::{
    config::{launch::AppLaunchConfig, Config},
    views::{components, utils::egui::measure_widget_text},
};

use super::Tab;

pub struct SettingsTab {}

impl SettingsTab {
    pub const fn new() -> Self {
        Self {}
    }

    fn render(&mut self, gui_config: &mut Config, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            // TODO: calculate instead of hardcoding
            if ui.available_width() > 500. {
                self.render_split_horizontally(gui_config, ui);
            } else {
                self.render_split_vertically(gui_config, ui);
            }
        });
    }

    fn render_split_horizontally(&mut self, gui_config: &mut Config, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            let space = ui.available_size();
            let button_size = space / 2.;
            
            ui.vertical(|ui| {
                ui.set_max_width(space.x / 2.);
                ui.vertical_centered(|ui| {
                    self.render_visual_settings(ui, gui_config, button_size);
                });
            });
            ui.separator();

            ui.vertical_centered(|ui| {
                self.render_behaviour_settings(ui, gui_config, button_size);
            });
        });
    }

    fn render_split_vertically(&mut self, gui_config: &mut Config, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            let mut button_size = ui.available_size();
            button_size.x *= 0.90; // horizontal margin
            button_size.y /= 2.;
            button_size.y = button_size.y.min(100.);

            ui.vertical_centered_justified(|ui| {
                self.render_visual_settings(ui, gui_config, button_size);
            });
            ui.separator();

            ui.vertical_centered_justified(|ui| {
                self.render_behaviour_settings(ui, gui_config, button_size);
            });
        });
    }

    fn render_visual_settings(&mut self, ui: &mut Ui, gui_config: &mut Config, button_size: Vec2) {
        ui.heading("Visual");

        render_switch_theme_button(gui_config, ui, button_size);
        
        render_font_size_slider(gui_config, ui, button_size);
    }

    fn render_behaviour_settings(&mut self, ui: &mut Ui, gui_config: &mut Config, button_size: Vec2) {
        ui.heading("Behaviour");

        render_close_window_when_game_loaded_checkbox(ui, button_size, gui_config);

        render_close_window_when_game_closes_checkbox(gui_config, ui, button_size);
    }
}

fn render_font_size_slider(
    gui_config: &mut Config,
    ui: &mut eframe::egui::Ui,
    space: eframe::epaint::Vec2,
) {
    let text = "Font size";
    
    let text_width = measure_widget_text(ui, text).x;
    ui.style_mut().spacing.slider_width = space.x - 5. - text_width;

    let slider = Slider::new(&mut gui_config.font_size, 10.0..=30.0)
        .step_by(1.0)
        .fixed_decimals(0)
        .text(text)
        .trailing_fill(true);
    if ui.add(slider).changed() {
        gui_config.update_text_styles(ui.ctx());
    }
}

fn render_close_window_when_game_loaded_checkbox(
    ui: &mut eframe::egui::Ui,
    space: eframe::epaint::Vec2,
    gui_config: &mut Config,
) {
    if components::checkbox(
        &mut gui_config.close_window_when_game_loaded,
        "Close this window when the game is loaded",
        ui,
        space,
        TextStyle::Body,
    ) {
        _ = gui_config.save_bepinex_toml_cfg_file();
    }
}

fn render_close_window_when_game_closes_checkbox(
    gui_config: &mut Config,
    ui: &mut eframe::egui::Ui,
    space: eframe::epaint::Vec2,
) {
    let close_window_when_game_closes = &mut gui_config
        .close_window_when_game_closes
        .load(Ordering::Relaxed);

    if components::checkbox(
        close_window_when_game_closes,
        "Close this window when the game closes",
        ui,
        space,
        TextStyle::Button,
    ) {
        gui_config
            .close_window_when_game_closes
            .store(*close_window_when_game_closes, Ordering::Relaxed);

        _ = gui_config.save_bepinex_toml_cfg_file();
    }
}

fn render_switch_theme_button(
    gui_config: &mut Config,
    ui: &mut eframe::egui::Ui,
    space: eframe::epaint::Vec2,
) {
    let is_dark_mode = gui_config.dark_mode;
    let text = if is_dark_mode { "Switch to light theme 🌞" } else { "Switch to dark theme 🌙" };
    let emoji = if is_dark_mode { "🌞" } else { "🌙" };

    if components::button_responsive_text(text, emoji, ui, space).clicked() {
        gui_config.dark_mode = !gui_config.dark_mode;
        gui_config.theme_just_changed = true;
    }
}

impl Tab for SettingsTab {
    fn name(&self) -> &str {
        "Settings"
    }

    fn update_top_panel(
        &mut self,
        _data: &AppLaunchConfig,
        _gui_config: &mut Config,
        _ui: &mut eframe::egui::Ui,
    ) {
    }

    fn update(
        &mut self,
        _data: &AppLaunchConfig,
        gui_config: &mut Config,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        self.render(gui_config, ctx);
    }
}

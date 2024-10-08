use crossbeam_channel::Receiver;

use eframe::{
    egui::{self, CentralPanel, Context, Layout, ScrollArea, TopBottomPanel, RichText},
    emath::Align,
};

use crate::{
    app,
    config::{launch::AppLaunchConfig, Config},
    data::bepinex_mod::BepInExMod,
    views::utils::egui::measure_widget_text,
};

use super::Tab;

pub struct GeneralTab {
    mod_receiver: Receiver<BepInExMod>,
    mods: Vec<BepInExMod>,
}

impl GeneralTab {
    pub fn new(mods_receiver: Receiver<BepInExMod>) -> Self {
        Self {
            mod_receiver: mods_receiver,
            mods: Vec::new(),
        }
    }

    fn render_footer(&mut self, data: &AppLaunchConfig, ctx: &Context) {
        TopBottomPanel::bottom("general_footer").show(ctx, |ui| {
            ui.add_space(25.0);

            app::BepInExGUI::render_useful_buttons_footer( 
                ui,
                data.game_folder_full_path(),
                data.bepinex_log_output_file_full_path(),
                data.target_process_id(),
            );
        });
    }

    fn render(&mut self, gui_config: &Config, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.mods.is_empty() {
                ui.vertical_centered_justified(|ui| {
                    ui.style_mut().wrap = Some(false);
                    let loading_text = RichText::new("Loading ⌛").heading();
                    let text_size = measure_widget_text(ui, loading_text.clone());
                    ui.add_space(ui.available_height() / 2. - text_size.y);
                    ui.label(loading_text);
                });
            } else {
                ui.spacing_mut().scroll_bar_width = 16.;
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.render_mods(gui_config, ui);
                    });
            }
        });
    }

    fn render_mods(&self, _gui_config: &Config, ui: &mut egui::Ui) {
        for mod_ in self.mods.as_slice() {
            ui.label(mod_.to_string());
        }
    }

    fn update_mod_receiver(&mut self) {
        if let Ok(mod_) = self.mod_receiver.try_recv() {
            self.mods.push(mod_);
        }
    }
}

impl Tab for GeneralTab {
    fn name(&self) -> &str {
        "General"
    }

    fn update_top_panel(
        &mut self,
        data: &AppLaunchConfig,
        _gui_config: &mut Config,
        ui: &mut eframe::egui::Ui,
    ) {
        egui::menu::bar(ui, move |ui| {
            // controls
            ui.with_layout(Layout::left_to_right(Align::default()), |ui| {
                let target_is_loading_text = format!(
                    "Modded {} is loading, you can close this window at any time.",
                    data.target_name()
                );
                ui.label(target_is_loading_text);
            });
        });

        let loaded_mod_count = self.mods.len();
        let loaded_mods_text = format!("Loaded Mods: {loaded_mod_count}");
        ui.label(loaded_mods_text);
    }

    fn update(
        &mut self,
        data: &AppLaunchConfig,
        gui_config: &mut Config,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        self.update_mod_receiver();

        self.render_footer(data, ctx);

        self.render(gui_config, ctx);
    }
}

use std::path::PathBuf;

use eframe::{
    self,
    egui::{include_image, Context, Image, TextStyle, TopBottomPanel, Ui, Visuals},
    emath::Vec2,
};
use sysinfo::Pid;

use crate::{
    app::BepInExGUI,
    backend::{file_explorer_utils, thunderstore},
    data::bepinex_log,
    views::components::button_responsive_img_widget,
};

use self::components::{button_responsive_img, button};

pub mod components;
pub mod disclaimer;
pub mod tabs;
pub mod utils;

impl BepInExGUI {
    pub(crate) fn view_update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if self.config.theme_just_changed {
            if self.config.dark_mode {
                ctx.set_style(self.dark_theme.clone());
            } else {
                ctx.set_visuals(Visuals::light());
            }
            self.config.update_text_styles(ctx);

            self.config.theme_just_changed = false;
        }

        if self.config.first_time {
            self.show_first_time_disclaimer(ctx);
        } else {
            self.render_header(ctx, frame);

            let tab = &mut self.tabs[self.config.selected_tab_index];

            tab.update(&self.app_launch_config, &mut self.config, ctx, frame);
        }
    }

    fn show_first_time_disclaimer(&mut self, ctx: &Context) {
        disclaimer::show(&mut self.config, &mut self.disclaimer, ctx);
    }

    fn render_header(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut button_size = ui.available_size() / 3.;
                button_size.y += 25.;

                ui.spacing_mut().item_spacing.x = 1.;
                ui.spacing_mut().item_spacing.y = 1.;

                for (i, tab) in self.tabs.iter().enumerate() {
                    if button(tab.name(), ui, button_size, TextStyle::Heading).clicked() {
                        self.config.selected_tab_index = i;
                    }
                } 
            });

            ui.add_space(10.);

            if !self.config.first_time_console_disclaimer {
                self.tabs[self.config.selected_tab_index].update_top_panel(
                    &self.app_launch_config,
                    &mut self.config,
                    ui,
                );
                ui.add_space(10.);
            }
        });
    }

    pub fn render_useful_buttons_footer(
        ui: &mut Ui,
        game_folder_full_path: &PathBuf,
        bepinex_log_output_file_full_path: &PathBuf,
        target_process_id: Pid,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(ui.available_width() / 8., 0.);
            let mut avail_space = ui.available_size();
            avail_space.x -= ui.spacing().item_spacing.x * 2.;
            let button_size = Vec2::new(avail_space.x / 3., avail_space.y);
            
            ui.add_space(0.); // insert item_spacing before first element
            render_open_game_folder_button(ui, button_size, game_folder_full_path);
            
            render_copy_log_file_button(ui, button_size, bepinex_log_output_file_full_path);
            
            render_open_modding_discord_button(ui, button_size, target_process_id);
        });
        ui.add_space(25.0);
    }
}

fn render_open_game_folder_button(ui: &mut Ui, button_size: Vec2, game_folder_full_path: &PathBuf) {
    let text: &str = "Open Game Folder";
    let img = Image::new(include_image!(
        "../../assets/icons/fa-regular folder-open.png"
    ))
    .tint(ui.visuals().text_color());

    if button_responsive_img(text, img, ui, button_size, TextStyle::Button).clicked() {
        file_explorer_utils::open_path_in_explorer(game_folder_full_path);
    }
}

fn render_copy_log_file_button(
    ui: &mut Ui,
    button_size: Vec2,
    bepinex_log_output_file_full_path: &PathBuf,
) {
    let text: &str = "Copy Log File";
    let img = Image::new(include_image!(
        "../../assets/icons/fa-regular clipboard.png"
    ))
    .tint(ui.visuals().text_color());

    if button_responsive_img(text, img, ui, button_size, TextStyle::Button).clicked() {
        bepinex_log::file::open_file_explorer_to_file_and_zip_it_if_needed(
            bepinex_log_output_file_full_path,
            "zipped_log.zip",
        );
    }
}

fn render_open_modding_discord_button(ui: &mut Ui, button_size: Vec2, target_process_id: Pid) {
    static mut HAS_DISCORD: bool = true;

    let text: &str = "Modding Discord";
    let img = Image::new(include_image!("../../assets/icons/discord.png"))
        .tint(ui.visuals().text_color());

    let button = button_responsive_img_widget(text, img, ui, button_size, TextStyle::Button);

    unsafe {
        let button_clicked = ui
            .add_enabled_ui(HAS_DISCORD, |ui| {
                ui.add_sized(button_size, button)
                    .on_disabled_hover_text("No modding Discord found")
                    .clicked()
            })
            .inner;
        if button_clicked {
            HAS_DISCORD = thunderstore::api::open_modding_discord(target_process_id);
        }
    }
}

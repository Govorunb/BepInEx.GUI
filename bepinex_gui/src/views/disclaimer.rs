use std::time::SystemTime;

use eframe::egui::{Context, Label, Window};

use crate::config::Config;

#[derive(Debug)]
pub struct Disclaimer {
    pub first_time_showing_it: bool,
    pub time_when_disclaimer_showed_up: Option<SystemTime>,
}

impl Default for Disclaimer {
    fn default() -> Self {
        Self {
            first_time_showing_it: true,
            time_when_disclaimer_showed_up: None,
        }
    }
}

pub fn show(config: &mut Config, disclaimer: &mut Disclaimer, ctx: &Context) {
    Window::new("Disclaimer")
        .collapsible(false)
        .resizable(false)
        .fixed_size(ctx.available_rect().size())
        .movable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add(
                Label::new(
r#"The console is now disabled by default.

If you notice issues with a mod while playing:

    - Head to the Modding Discord by clicking on the "Modding Discord" button.
    - Post the log file by copying it to clipboard through the "Copy Log File" button.
    - Wait for help.

For mod developers that like the old conhost console, you can enable it back by opening the BepInEx/config/BepInEx.cfg and setting to true the "Enables showing a console for log output." config option."#).wrap(true));

                if disclaimer.first_time_showing_it {
                    disclaimer.time_when_disclaimer_showed_up = Some(SystemTime::now());
                    disclaimer.first_time_showing_it = false;
                }

                if let Ok(elapsed_) = disclaimer.time_when_disclaimer_showed_up.unwrap().elapsed() {
                    let elapsed = elapsed_.as_secs() as i64;
                    const NEEDED_TIME_BEFORE_CLOSABLE:i64 = 9;
                    let can_close = elapsed > NEEDED_TIME_BEFORE_CLOSABLE;
                    if can_close {
                        ui.centered_and_justified(|ui| {
                            if ui.button("Ok").clicked() {
                                config.first_time = false;
                            }
                        });
                    }
                    else {
                        ui.centered_and_justified(|ui| {
                            ui.label(((NEEDED_TIME_BEFORE_CLOSABLE + 1) - elapsed).to_string());
                        });
                    }
                }
            });
        });
}

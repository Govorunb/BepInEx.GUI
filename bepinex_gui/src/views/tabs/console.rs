use clipboard::*;
use crossbeam_channel::Receiver;
use eframe::{egui::*, *};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

use crate::{
    backend::process,
    config::{launch::AppLaunchConfig, Config},
    data::{
        bepinex_log::{BepInExLogEntry, LogLevel},
        bepinex_mod::BepInExMod,
    },
    views::{self, disclaimer::Disclaimer, components::button},
};

use super::Tab;

struct LogSelection {
    pub button_currently_down: bool,
    pub button_just_got_down: bool,
    pub index_of_first_selected_log: usize,
    pub index_of_last_selected_log: usize,
    pub index_of_last_unselected_log: usize,
    pub cursor_pos_when_button_was_pressed: Option<Pos2>,
}

impl LogSelection {
    fn update_pointer_state(&mut self, ctx: &Context) {
        self.button_currently_down = ctx.input(|i| i.pointer.primary_down());
        self.button_just_got_down = ctx.input(|i| i.pointer.primary_pressed());
        self.cursor_pos_when_button_was_pressed = ctx.input(|i| i.pointer.press_origin());
    }

    fn update_selection(
        &mut self,
        ui_log_entry: &Response,
        clip_rect: &Rect,
        ui: &Ui,
        log_index: usize,
    ) {
        if self.button_currently_down {
            let mut log_rect = ui_log_entry.rect;
            // make it so that just selecting anywhere within the log line work
            log_rect.max.x = clip_rect.max.x;
            // make it so that there is no dead space between the log entries for log selection purposes
            log_rect.min.y += 4.;
            log_rect.max.y += 4.;

            if ui.rect_contains_pointer(log_rect) {
                if self.button_just_got_down {
                    let is_a_new_pressed_log = self.index_of_first_selected_log != log_index;
                    if is_a_new_pressed_log {
                        self.index_of_first_selected_log = log_index;
                        self.index_of_last_selected_log = log_index;
                    } else {
                        // we just pressed the same button, unselect all
                        self.index_of_first_selected_log = usize::MAX;
                        self.index_of_last_selected_log = usize::MAX;

                        // we remember the last unselected button,
                        // because the user may still hold the button and it may instantly reselect it the next frame
                        self.index_of_last_unselected_log = log_index;
                    }
                } else {
                    // user is holding the button, and hovering a log entry

                    let user_is_holding_and_selecting_a_new_log =
                        self.index_of_last_unselected_log != log_index;
                    if user_is_holding_and_selecting_a_new_log {
                        self.index_of_last_selected_log = log_index;

                        // fix an edge case where the user just unselected a log,
                        // and is now selecting the one just above or just below,
                        // all within the same keypress / kept holding
                        if self.index_of_first_selected_log == usize::MAX {
                            self.index_of_first_selected_log = log_index;
                        }
                    }
                }
            } else if self.button_just_got_down {
                self.index_of_first_selected_log = usize::MAX;
                self.index_of_last_selected_log = usize::MAX;
            }
        }
    }
}

struct Filter {
    text: String,
    text_lowercase: String,
    pub selected_index_in_mods_combo_box: usize,
}

struct Scroll {
    last_log_count: usize,
    pending_scroll: Option<Vec2>,
}

pub struct ConsoleTab {
    disclaimer: Disclaimer,
    log_selection: LogSelection,
    filter: Filter,
    scroll: Scroll,
    target_process_paused: bool,
    mod_receiver: Receiver<BepInExMod>,
    mods: Vec<BepInExMod>,
    log_receiver: Receiver<BepInExLogEntry>,
    logs: Vec<BepInExLogEntry>,
    should_exit_app: Arc<AtomicBool>,
    log_heights: HashMap<usize, f32>,
}

impl ConsoleTab {
    pub fn new(
        mod_receiver: Receiver<BepInExMod>,
        log_receiver: Receiver<BepInExLogEntry>,
        should_exit_app: Arc<AtomicBool>,
    ) -> Self {
        Self {
            disclaimer: Disclaimer {
                first_time_showing_it: true,
                time_when_disclaimer_showed_up: None,
            },
            log_selection: LogSelection {
                button_currently_down: false,
                button_just_got_down: false,
                index_of_first_selected_log: usize::MAX,
                index_of_last_selected_log: usize::MAX,
                index_of_last_unselected_log: usize::MAX,
                cursor_pos_when_button_was_pressed: None,
            },
            filter: Filter {
                text: Default::default(),
                text_lowercase: Default::default(),
                selected_index_in_mods_combo_box: 0,
            },
            scroll: Scroll {
                last_log_count: 0,
                pending_scroll: None,
            },
            target_process_paused: false,
            mod_receiver,
            mods: vec![BepInExMod::new("", "")],
            log_receiver,
            logs: vec![],
            should_exit_app,
            log_heights: HashMap::new(),
        }
    }

    fn render(&mut self, gui_config: &Config, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.logs.is_empty() {
                render_loading_text(ui);
            } else {
                self.render_console_scroll_area(ui, gui_config);
            }
        });
    }

    fn render_console_scroll_area(&mut self, ui: &mut Ui, gui_config: &Config) {
        ui.spacing_mut().scroll_bar_width = 16.;

        let scroll_area = ScrollArea::vertical()
            .drag_to_scroll(false)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if self.log_selection.button_just_got_down {
                    self.logs.iter_mut().for_each(|log| log.is_selected = false);
                }

                self.render_logs(gui_config, ui);

                if let Some(scroll) = self.scroll.pending_scroll {
                    ui.scroll_with_delta(scroll);
                    self.scroll.pending_scroll = None;
                }
            });

        self.auto_scroll_to_selection(&scroll_area, ui);
    }

    fn auto_scroll_to_selection(
        &mut self,
        scroll_area: &scroll_area::ScrollAreaOutput<()>,
        ui: &mut Ui,
    ) {
        if let Some(cursor_pos) = self.log_selection.cursor_pos_when_button_was_pressed {
            if self.log_selection.button_currently_down
                && scroll_area.inner_rect.contains(cursor_pos)
            {
                self.scroll.pending_scroll =
                    views::utils::egui::scroll_when_trying_to_select_stuff_above_or_under_rect(
                        ui,
                        scroll_area.inner_rect,
                    );
            }
        }
    }

    fn kill_gui_and_target(&mut self, data: &AppLaunchConfig) {
        process::kill(data.target_process_id(), || {
            tracing::info!("Exiting because Command + F5 was pressed.");
            self.should_exit_app.store(true, Ordering::Relaxed);
        });
    }

    fn render_logs(&mut self, gui_config: &Config, ui: &mut eframe::egui::Ui) {
        let clip_rect = ui.painter().clip_rect();

        let log_count = self.logs.len();
        for i in 0..log_count {
            Self::render_log(
                &mut self.log_heights,
                &mut self.filter,
                gui_config,
                i,
                ui,
                &clip_rect,
                &mut self.log_selection,
                &mut self.logs[i],
            );

            self.logs[i].is_selected = is_between(
                i,
                self.log_selection.index_of_first_selected_log,
                self.log_selection.index_of_last_selected_log,
            );
        }

        if gui_config.log_auto_scroll_to_bottom
            && self.scroll.last_log_count != log_count
            && !self.log_selection.button_just_got_down
        {
            ui.scroll_with_delta(Vec2::new(0., f32::NEG_INFINITY));
            self.scroll.last_log_count = log_count;
        }
    }

    fn update_copy_logs_to_clipboard(&mut self, ctx: &Context) {
        if ctx.input(|i| i.modifiers.command) && ctx.input(|i| i.key_pressed(Key::C)) {
            if let Ok(ctx_) = ClipboardProvider::new() {
                let mut ctx: ClipboardContext = ctx_;

                let selected_logs: Vec<String> = self
                    .logs
                    .iter()
                    .filter(|x| x.is_selected)
                    .map(|x| x.data().to_string())
                    .collect();

                let selected_logs_string = selected_logs.join("\n");

                match ctx.set_contents(selected_logs_string) {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("Failed copying logs to clipboard: {}", err);
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_log(
        log_heights: &mut HashMap<usize, f32>,
        filter: &mut Filter,
        gui_config: &Config,
        i: usize,
        ui: &mut Ui,
        clip_rect: &Rect,
        log_selection: &mut LogSelection,
        log: &mut BepInExLogEntry,
    ) {
        if log.level() > gui_config.log_level_filter {
            return;
        }

        if !Self::does_log_match_text_filter(&filter.text_lowercase, log) {
            return;
        }

        let pos_before_log = ui.next_widget_position();

        let log_render_decision = make_log_render_decision(
            log_heights,
            i,
            pos_before_log,
            ui.ctx().input(|i| i.screen_rect),
            ui,
        );
        if log_render_decision == LogRenderDecision::SkipAndFakeRender {
            return;
        }

        let log_color = get_color_from_log_level(log, ui.style().visuals.strong_text_color(), gui_config);

        let ui_log_entry = make_ui_log_entry(ui, log, log_color);

        let pos_after_log = ui.next_widget_position();

        if log_render_decision == LogRenderDecision::RenderAndCacheHeight {
            let log_height = pos_after_log.y - pos_before_log.y;
            log_heights.insert(i, log_height);
        }

        log_selection.update_selection(&ui_log_entry, clip_rect, ui, i);
    }

    fn does_log_match_text_filter(text_filter_lowercase: &String, log: &BepInExLogEntry) -> bool {
        log.data_lowercase().contains(text_filter_lowercase)
    }

    fn render_footer(&mut self, data: &AppLaunchConfig, gui_config: &mut Config, ctx: &Context) {
        TopBottomPanel::bottom("console_footer").show(ctx, |ui| {
            ui.add_space(2.0);

            ui.horizontal(|ui| {
                // TODO: calculate instead of hardcoding
                if ui.available_width() > 250. {
                    ui.label(RichText::new("Log Level: ").small());
                }

                let log_level_text = gui_config.log_level_filter.to_string();
                ui.add(
                    Slider::new(
                        &mut gui_config.log_level_filter,
                        LogLevel::Fatal..=LogLevel::All,
                    )
                    .show_value(false)
                    .text(log_level_text),
                );
                
                render_auto_scroll_to_bottom_checkbox(ui, gui_config);
            });

            views::BepInExGUI::render_useful_buttons_footer(
                ui,
                data.game_folder_full_path(),
                data.bepinex_log_output_file_full_path(),
                data.target_process_id(),
            );
        });
    }

    fn render_console_first_time_disclaimer(&mut self, ctx: &Context, gui_config: &mut Config) {
        CentralPanel::default().show(ctx, |_| {
            Window::new("Console Disclaimer")
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.heading(
r#"The console is meant to be used by mod developers.
If any of your mods is malfunctioning and that you wish to receive help in the #tech-support channel of the discord:
Please use the buttons below and use the "Copy Log File" button, and drag and drop it in the #tech-support channel."#);

                    if self.disclaimer.first_time_showing_it {
                        self.disclaimer.time_when_disclaimer_showed_up = Some(SystemTime::now());
                        self.disclaimer.first_time_showing_it = false;
                    }

                    if let Ok(elapsed_) = self
                        .disclaimer
                        .time_when_disclaimer_showed_up
                        .unwrap()
                        .elapsed()
                    {
                        let elapsed = elapsed_.as_secs() as i64;
                        if 9 - elapsed >= 0 {
                            ui.label((10 - elapsed).to_string());
                        } else if ui.button("Ok").clicked() {
                            gui_config.first_time_console_disclaimer = false;
                        }
                    }
                });
        });
    }

    fn render_kill_gui_and_game_button(
        &mut self,
        ui: &mut Ui,
        data: &AppLaunchConfig,
    ) {
        let kill_game_btn_text = "Close Game & GUI";
        let kill_game_btn_size = views::utils::egui::compute_text_size(
            ui,
            kill_game_btn_text,
            Some(TextStyle::Heading),
            false,
        ).unwrap();

        if button(kill_game_btn_text, ui, kill_game_btn_size, TextStyle::Heading)
            .clicked()
        {
            self.kill_gui_and_target(data);
        }
    }

    fn render_pause_game_button(&mut self, ui: &mut Ui, data: &AppLaunchConfig) {
        let pause_game_btn_text = if self.target_process_paused {
            "Resume Game"
        } else {
            "Pause Game"
        };
        let pause_game_btn_size = views::utils::egui::compute_text_size(
            ui,
            pause_game_btn_text,
            Some(TextStyle::Heading),
            false,
        ).unwrap();

        if button(pause_game_btn_text, ui, pause_game_btn_size, TextStyle::Heading)
            .clicked()
        {
            let target_process_id = data.target_process_id();

            if self.target_process_paused {
                self.target_process_paused = !process::resume(target_process_id);
            } else {
                self.target_process_paused = process::suspend(target_process_id);
            }
        }
    }

    fn render_log_text_filter_input(&mut self, ui: &mut Ui, mods_combo_box: &Response) {
        if ui
            .add_sized(
                mods_combo_box.rect.size(),
                TextEdit::singleline(&mut self.filter.text)
                    .text_color(ui.style().visuals.strong_text_color())
                    .hint_text("Filter Text"),
            )
            .changed()
        {
            self.filter.text_lowercase = self.filter.text.to_lowercase();
        }
    }

    fn render_log_mod_filter(&mut self, ui: &mut Ui) -> Response {
        let mods_combo_box = ComboBox::from_id_source("combo_box_mods_log_filter")
            .width(200.)
            .show_index(
                ui,
                &mut self.filter.selected_index_in_mods_combo_box,
                self.mods.len(),
                |i| self.mods[i].name(),
            );

        if mods_combo_box.changed() {
            self.filter.text = if self.filter.selected_index_in_mods_combo_box == 0 {
                String::new()
            } else {
                self.mods[self.filter.selected_index_in_mods_combo_box]
                    .name()
                    .to_string()
            };

            self.filter.text_lowercase = self.filter.text.to_lowercase();
        }
        mods_combo_box
    }
}

#[derive(PartialEq)]
enum LogRenderDecision {
    SkipAndFakeRender,
    RenderAndCacheHeight,
    RenderNormally,
}

fn make_log_render_decision(
    log_heights: &mut HashMap<usize, f32>,
    i: usize,
    pos_before_log: Pos2,
    screen_rect: Rect,
    ui: &mut Ui,
) -> LogRenderDecision {
    if let Some(height_) = log_heights.get(&i) {
        let height = *height_;
        let after_the_bottom = pos_before_log.y > screen_rect.max.y;
        let before_the_top = pos_before_log.y + height < 0.0;

        if after_the_bottom || before_the_top {
            // Don't actually render, just make space for scrolling purposes
            ui.add_space(height);
            return LogRenderDecision::SkipAndFakeRender;
        }
    } else {
        return LogRenderDecision::RenderAndCacheHeight;
    }

    LogRenderDecision::RenderNormally
}

fn render_loading_text(ui: &mut Ui) {
    ui.centered_and_justified(|ui| {
        let loading_text = "Loading ⌛";
        ui.heading(loading_text);
    });
}

fn make_ui_log_entry(ui: &mut Ui, log: &mut BepInExLogEntry, log_color: Color32) -> Response {
    let ui_log_entry = ui.add(SelectableLabel::new(
        log.is_selected,
        RichText::new(log.data()).color(log_color),
    ));
    ui_log_entry
}

const ORANGE: Color32 = Color32::from_rgb(255, 128, 0);
fn get_color_from_log_level(log: &mut BepInExLogEntry, info_log_color: Color32, gui_config: &Config) -> Color32 {
    // TODO: put the colors in a style
    match log.level() {
        LogLevel::None | LogLevel::Fatal => if gui_config.dark_mode {Color32::RED} else {Color32::DARK_RED},
        LogLevel::Error => if gui_config.dark_mode {Color32::LIGHT_RED} else {Color32::RED},
        LogLevel::Warning => if gui_config.dark_mode {Color32::YELLOW} else {ORANGE},
        LogLevel::Message => info_log_color,
        LogLevel::Info => if gui_config.dark_mode {Color32::LIGHT_GRAY} else {Color32::DARK_GRAY},
        LogLevel::Debug => Color32::GRAY,
        LogLevel::All => info_log_color,
    }
}

impl Tab for ConsoleTab {
    fn name(&self) -> &str {
        "Console"
    }

    fn update_top_panel(
        &mut self,
        data: &AppLaunchConfig,
        _gui_config: &mut Config,
        ui: &mut eframe::egui::Ui,
    ) {
        egui::menu::bar(ui, move |ui| {
            // controls
            ui.with_layout(Layout::left_to_right(Align::Center), move |ui| {
                ui.label(RichText::new("Log Filtering: ").small());
                let mods_combo_box = self.render_log_mod_filter(ui);
                self.render_log_text_filter_input(ui, &mods_combo_box);
                
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
    
                    self.render_pause_game_button(ui, data);
    
                    self.render_kill_gui_and_game_button(ui, data);
                });
            });
        });
    }

    fn update(
        &mut self,
        data: &AppLaunchConfig,
        gui_config: &mut Config,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        self.log_selection.update_pointer_state(ctx);

        self.update_mod_receiver();
        self.update_log_receiver();

        if gui_config.first_time_console_disclaimer {
            self.render_console_first_time_disclaimer(ctx, gui_config);
        } else {
            self.render_footer(data, gui_config, ctx);

            self.render(gui_config, ctx);

            self.update_copy_logs_to_clipboard(ctx);
        }

        if ctx.input(|i| i.modifiers.command) && ctx.input(|i| i.key_pressed(Key::F5)) {
            self.kill_gui_and_target(data);
        }
    }
}

fn render_auto_scroll_to_bottom_checkbox(ui: &mut Ui, gui_config: &mut Config) {
    let text = "Auto Scroll to Bottom";

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.checkbox(&mut gui_config.log_auto_scroll_to_bottom, text);
    });
}

impl ConsoleTab {
    fn update_mod_receiver(&mut self) {
        if let Ok(mod_) = self.mod_receiver.try_recv() {
            self.mods.push(mod_);
        }
    }

    fn update_log_receiver(&mut self) {
        // loop until the channel is emptied
        // if we don't do that the maximum amount of log received is
        // tied to the framerate of the GUI
        loop {
            match self.log_receiver.try_recv() {
                Ok(log) => {
                    self.logs.push(log);
                }
                Err(err) => match err {
                    crossbeam_channel::TryRecvError::Disconnected
                    | crossbeam_channel::TryRecvError::Empty => {
                        break;
                    }
                },
            }
        }
    }
}

fn is_between<T: Ord + std::marker::Copy>(value: T, bound1: T, bound2: T) -> bool {
    let lower_bound = std::cmp::min(bound1, bound2);
    let upper_bound = std::cmp::max(bound1, bound2);

    value >= lower_bound && value <= upper_bound
}

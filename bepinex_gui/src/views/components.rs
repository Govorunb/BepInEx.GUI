use eframe::egui::*;

use super::utils::egui::measure_widget_text;

pub fn button(text: impl Into<WidgetText>, ui: &mut Ui, button_size: Vec2) -> Response {
    ui.add_sized(button_size, Button::new(text))
}

/// Button whose text gets replaced with `short_text` (e.g. an emoji) when there's not enough space to display it in full.
pub fn button_responsive_text(
    text: impl Into<WidgetText> + Clone,
    short_text: impl Into<WidgetText>,
    ui: &mut Ui,
    space: Vec2,
) -> Response {
    let text_size = measure_widget_text(ui, text.clone());
    
    let fits = space.x > text_size.x && space.y > text_size.y;
    
    if fits {
        button(text, ui, space)
    } else {
        button(short_text, ui, space).on_hover_text(text)
    }
}

pub fn button_responsive_text_widget<'a>(
    text: impl Into<WidgetText> + Clone,
    short_text: impl Into<WidgetText>,
    ui: &mut Ui,
    space: Vec2,
) -> Button<'a> {
    let text_size = measure_widget_text(ui, text.clone());
    
    let fits = space.x > text_size.x && space.y > text_size.y;
    if fits {
        Button::new(text)
    } else {
        Button::new(short_text)
    }
}

pub fn checkbox(
    bool_ref: &mut bool,
    text: &str,
    ui: &mut Ui,
    button_size: Vec2,
    text_style: TextStyle,
) -> bool {
    ui.add_sized(
        button_size,
        Checkbox::new(bool_ref, RichText::new(text).text_style(text_style)),
    )
    .clicked()
}

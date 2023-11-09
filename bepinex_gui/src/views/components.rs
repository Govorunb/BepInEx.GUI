use eframe::egui::*;

use super::utils::egui::compute_text_size;

pub fn button(text: &str, ui: &mut Ui, button_size: Vec2, text_style: TextStyle) -> Response {
    ui.add_sized(button_size, button_widget(text, text_style))
}

pub fn button_widget<'a>(text: &str, text_style: TextStyle) -> Button<'a> {
    Button::new(RichText::new(text).text_style(text_style))
}

#[allow(dead_code)]
pub fn colored_button(
    text: &str,
    ui: &mut Ui,
    button_size: Vec2,
    text_style: TextStyle,
    fill_color: Option<Color32>,
) -> bool {
    let mut btn = Button::new(RichText::new(text).text_style(text_style));
    if let Some(color) = fill_color {
        btn = btn.fill(color);
        btn = btn.stroke(Stroke::default());
    }

    ui.add_sized(button_size, btn).clicked()
}

pub fn img_button<'a>(
    image: impl Into<Image<'a>>,
    tooltip: &str,
    ui: &mut Ui,
    button_size: Vec2,
) -> Response {
    ui.add_sized(button_size, Button::image(image))
        .on_hover_text(tooltip)
}

// TODO: replace with FontAwesome or something

/// Button that gets replaced with an image when there's not enough space to display the text
pub fn button_responsive_img<'a>(
    text: &str,
    image: impl Into<Image<'a>>,
    ui: &mut Ui,
    space: Vec2,
    text_style: TextStyle,
) -> Response {
    let text_size = compute_text_size(ui, text, Some(text_style.clone()), true)
        .unwrap_or_else(|_| panic!("Unknown TextStyle {text_style}"));

    if space.x > text_size.x && space.y > text_size.y {
        button(text, ui, space, text_style)
    } else {
        img_button(image, text, ui, space)
    }
}

pub fn button_responsive_img_widget<'a>(
    text: &'a str,
    image: impl Into<Image<'a>>,
    ui: &mut Ui,
    space: Vec2,
    text_style: TextStyle,
) -> Button<'a> {
    let text_size = compute_text_size(ui, text, Some(text_style.clone()), true)
        .unwrap_or_else(|_| panic!("Unknown TextStyle {text_style}"));

    if space.x > text_size.x && space.y > text_size.y {
        button_widget(text, text_style)
    } else {
        Button::image(image)
    }
}

/// Button whose text gets replaced with `short_text` (e.g. an emoji) when there's not enough space to display it in full.
pub fn button_responsive_text(
    text: &str,
    short_text: &str,
    ui: &mut Ui,
    space: Vec2,
    text_style: TextStyle,
) -> Response {
    let text_size = compute_text_size(ui, text, Some(text_style.clone()), true)
        .unwrap_or_else(|_| panic!("Unknown TextStyle {text_style}"));
    
    let fits = space.x > text_size.x && space.y > text_size.y;
    
    if fits {
        button(text, ui, space, text_style)
    } else {
        button(short_text, ui, space, text_style).on_hover_text(text)
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

use std::{error::Error, fmt::{Display, Formatter}};

use eframe::egui::*;

#[derive(Debug, Clone)]
pub struct TextStyleNotFoundError(TextStyle);

impl Error for TextStyleNotFoundError { }
impl Display for TextStyleNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown text style ({})", self.0)
    }
}

/// Returns the size of the given `text` as it would be painted with the given `text_style`.
/// 
/// This is safe to unwrap for known [`TextStyle`]s;
/// you only need to check for errors if you pass in [`TextStyle::Name`].
pub fn compute_text_size(
    ui: &mut Ui,
    text: &str,
    text_style: Option<TextStyle>,
    wrap: bool,
) -> Result<Vec2, TextStyleNotFoundError> {
    // the previous method for measuring text relied on calling ui.set_cursor()
    // after performing layout on an invisible label
    // unfortunately, it has at some point has become private
    // so now we have to dig the trench (ask to layout the text) ourselves
    let style = text_style.unwrap_or(TextStyle::Body);

    let wrap_width = if wrap {
        ui.available_width()
    } else {
        f32::INFINITY
    };

    let string: String = text.into();
    let font: FontId = ui.style().text_styles.get(&style)
        .ok_or(TextStyleNotFoundError(style))?
        .clone();
    let color = Color32::default();

    let galley = ui.fonts(|f| {
        if wrap {
            f.layout(string, font, color, wrap_width)
        } else {
            f.layout_no_wrap(string, font, color)
        }
    });
    Ok(galley.size())
}

pub fn scroll_when_trying_to_select_stuff_above_or_under_rect(
    ui: &mut Ui,
    rect: Rect,
) -> Option<Vec2> {
    if !ui.rect_contains_pointer(rect) {
        if let Some(interact_pos) = ui.input(|i| i.pointer.interact_pos()) {
            let mut scroll = Vec2::default();
            let dist = rect.bottom() - interact_pos.y;

            const SCROLL_SPEED: f32 = 0.25;

            if dist < 0. {
                scroll.y = dist;
                scroll.y *= SCROLL_SPEED;
            } else if dist > 0. {
                scroll.y = rect.top() - interact_pos.y;
                scroll.y *= SCROLL_SPEED;
            }

            return Some(scroll);
        }
    }

    None
}

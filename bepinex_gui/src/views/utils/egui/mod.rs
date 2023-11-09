use eframe::egui::*;

/// Returns the size of the given `text` as it would be painted.
///
/// If you want to manipulate UI behaviour like text wrapping, do the calculation inside a `ui.scope()`.
pub fn measure_widget_text(ui: &mut Ui, text: impl Into<WidgetText>) -> Vec2 {
    text.into()
        .into_galley(ui, None, ui.available_width(), FontSelection::Default)
        .size()
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

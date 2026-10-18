//! Touch-first layout for phones and tablets (the Android shell turns it on with [`enable`]).
//!
//! The desktop chrome (a menu bar of ten titles, a toolbox column, a 250–520 pt dock) does not fit a
//! 360 pt wide screen and its 24 pt controls are too small for a finger. With touch mode on:
//!
//! - **Top bar**: Menu button, document name (tap to switch documents), Undo, Redo.
//! - **Options bar** under it: the same tool options, scrolling sideways, taller.
//! - **Bottom bar**: every tool slot as a 48 pt button in a sideways-scrolling strip (tap the selected
//!   one again to open its flyout), plus the colour chips and the Panels button.
//! - **Sheet** above the bottom bar (a right-hand panel on wide tablets): Layers, Colour, Properties,
//!   History, Navigator and Character with their tabs, or the full menu tree as big rows.
//!
//! Everything still goes through commands, so behaviour is the desktop's. Hit targets are at least
//! 44–48 pt (points are dp on Android) and nothing is wider than the screen: strips scroll.

use egui::{Align, Align2, Color32, Frame, Id, Layout, Margin, Rect, RichText, Sense, Stroke, UiBuilder, pos2, vec2};
use serde_json::json;

use crate::dock::Group;
use crate::theme::Tokens;
use crate::{PhotocraftApp, icons};

/// Height of the top and bottom bars (points).
const BAR_H: f32 = 56.0;
/// Side of a square button (points).
const BTN: f32 = 48.0;
/// Height of a list row in the sheet and the menu (points).
const ROW_H: f32 = 48.0;
/// From this width up the sheet is a side panel instead of a bottom sheet.
const WIDE: f32 = 760.0;

fn flag_id() -> Id {
    Id::new("photocraft-touch-ui")
}

/// Is the touch layout on?
pub fn on(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(flag_id())).unwrap_or(false)
}

/// Turn the touch layout on or off. Call before `PhotocraftApp::setup_context` so the first frame
/// already has the bigger style; later calls take effect when the theme is next applied.
pub fn enable(ctx: &egui::Context, enabled: bool) {
    ctx.data_mut(|d| d.insert_temp(flag_id(), enabled));
    if enabled {
        ctx.global_style_mut(crate::theme::touch_tune);
        let style = ctx.global_style();
        ctx.set_style_of(egui::Theme::Light, std::sync::Arc::clone(&style));
        ctx.set_style_of(egui::Theme::Dark, style);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Sheet {
    Menu,
    Panel(Group),
}

fn sheet_id() -> Id {
    Id::new("touch-sheet")
}

fn current_sheet(ctx: &egui::Context) -> Option<Sheet> {
    ctx.data(|d| d.get_temp::<Option<Sheet>>(sheet_id())).flatten()
}

fn set_sheet(ctx: &egui::Context, s: Option<Sheet>) {
    ctx.data_mut(|d| d.insert_temp(sheet_id(), s));
}

fn menu_path_id() -> Id {
    Id::new("touch-menu-path")
}

fn color32(c: [f32; 4]) -> Color32 {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(b(c[0]), b(c[1]), b(c[2]), b(c[3]))
}

/// All the chrome of the touch layout, in the order egui needs (outer panels first).
pub fn chrome(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    top_bar(app, ui);
    if app.ui.panels.options_bar {
        crate::panels::options_bar(app, ui);
    }
    tool_bar(app, ui);
    sheet(app, ui);
}

// ----------------------------------------------------------------------------- top bar

fn hamburger(ui: &mut egui::Ui, selected: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(BTN, BTN), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, t.radius_sm, t.accent_soft);
    } else if resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(rect, t.radius_sm, t.hover);
    }
    let color = if selected { t.accent_text } else { t.icon };
    let c = rect.center();
    for dy in [-6.0, 0.0, 6.0] {
        ui.painter().line_segment([pos2(c.x - 9.0, c.y + dy), pos2(c.x + 9.0, c.y + dy)], Stroke::new(2.0, color));
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Menu")));
    resp
}

fn top_bar(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    egui::Panel::top("touch_top")
        .exact_size(BAR_H)
        .frame(Frame::NONE.fill(t.chrome).inner_margin(Margin::symmetric(6, 4)))
        .show(ui, |ui| {
            crate::direction::horizontal_centered(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let menu_open = current_sheet(&ctx) == Some(Sheet::Menu);
                if hamburger(ui, menu_open).clicked() {
                    ctx.data_mut(|d| d.remove::<Vec<String>>(menu_path_id()));
                    set_sheet(&ctx, if menu_open { None } else { Some(Sheet::Menu) });
                }
                // The document name takes what the two history buttons leave.
                let title_w = (ui.available_width() - 2.0 * BTN - 8.0).max(40.0);
                let (rect, resp) = ui.allocate_exact_size(vec2(title_w, BTN), Sense::click());
                let docs: Vec<(usize, String, bool)> = app
                    .session
                    .documents()
                    .iter()
                    .enumerate()
                    .map(|(i, d)| (i, d.doc.name.clone(), d.is_dirty()))
                    .collect();
                let active = app.session.active_index();
                let (name, dirty) = active
                    .and_then(|i| docs.iter().find(|d| d.0 == i))
                    .map(|d| (d.1.clone(), d.2))
                    .unwrap_or_else(|| ("PhotoCraft".to_string(), false));
                {
                    let mut child = ui.new_child(UiBuilder::new().max_rect(rect.shrink2(vec2(6.0, 0.0))).layout(Layout::top_down(Align::Min)));
                    child.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    child.spacing_mut().item_spacing.y = 0.0;
                    let label = if dirty { format!("{name}  •") } else { name };
                    child.add_space(if app.ui.status.is_empty() { 14.0 } else { 6.0 });
                    child.label(RichText::new(label).size(16.0).color(t.text));
                    if !app.ui.status.is_empty() {
                        let is_err = app.ui.status_error || app.ui.status.starts_with("Couldn");
                        child.label(RichText::new(&app.ui.status).size(12.0).color(if is_err { t.warning } else { t.text_faint }));
                    }
                }
                // With several documents open, tapping the name lists them.
                let mut switch: Option<usize> = None;
                if docs.len() > 1 {
                    egui::Popup::menu(&resp).close_behavior(egui::PopupCloseBehavior::CloseOnClick).show(|ui| {
                        ui.set_min_width(220.0);
                        for (i, name, dirty) in &docs {
                            let text = format!("{}{}{}", if Some(*i) == active { "✔ " } else { "    " }, name, if *dirty { "  •" } else { "" });
                            if ui.add_sized([ui.available_width(), ROW_H], egui::Button::new(text)).clicked() {
                                switch = Some(*i);
                            }
                        }
                    });
                }
                if let Some(i) = switch {
                    app.session.set_active(i);
                }
                if icons::button(ui, "undo-2", BTN, false, "Undo").clicked() {
                    let _ = app.run("edit.undo", json!({}));
                }
                if icons::button(ui, "redo-2", BTN, false, "Redo").clicked() {
                    let _ = app.run("edit.redo", json!({}));
                }
            });
        });
}

// ----------------------------------------------------------------------------- bottom bar

fn flyout_id() -> Id {
    Id::new("touch-tool-flyout")
}

fn tool_bar(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    egui::Panel::bottom("touch_tools")
        .exact_size(BAR_H + 8.0)
        .frame(Frame::NONE.fill(t.chrome).inner_margin(Margin::symmetric(6, 4)))
        .show(ui, |ui| {
            // Colour chips and Panels sit at the far end; the tools scroll in the rest.
            let gap = 6.0;
            let right_w = 2.0 * BTN + 2.0 * gap;
            let full = ui.available_rect_before_wrap();
            let left = Rect::from_min_max(full.min, pos2((full.max.x - right_w).max(full.min.x), full.max.y));
            let right = Rect::from_min_max(pos2((full.max.x - right_w).max(full.min.x), full.min.y), full.max);

            let hidden = app.session.prefs().toolbar.hidden.clone();
            let sections = crate::panels::visible_sections(&hidden);
            let mut tools_ui = ui.new_child(UiBuilder::new().max_rect(left).layout(Layout::left_to_right(Align::Center)));
            egui::ScrollArea::horizontal()
                .id_salt("touch-tools-strip")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .auto_shrink([false, false])
                .show(&mut tools_ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (slot_index, slot) in sections.iter().flatten() {
                        let key = Id::new(("tool-slot", *slot_index));
                        let tool = crate::panels::slot_tool(ui, app.ui.tool, slot, key);
                        let sel = slot.contains(&app.ui.tool);
                        let tip = tl!(tool.label()).to_string();
                        let resp = icons::button_with_icon_size_tip(ui, icons::tool_icon(tool), BTN, 26.0, sel, &tip, false);
                        if slot.len() > 1 {
                            crate::panels::paint_toolbar_flyout_indicator(ui, &t, &resp);
                        }
                        let open_here = ctx.data(|d| d.get_temp::<(Id, Rect)>(flyout_id())).is_some_and(|(k, _)| k == key);
                        if resp.clicked() {
                            if slot.len() > 1 && sel {
                                // The selected multi-tool slot again: its flyout.
                                if open_here {
                                    ctx.data_mut(|d| d.remove::<(Id, Rect)>(flyout_id()));
                                } else {
                                    ctx.data_mut(|d| d.insert_temp(flyout_id(), (key, resp.rect)));
                                }
                            } else {
                                app.ui.tool = tool;
                                ctx.data_mut(|d| d.remove::<(Id, Rect)>(flyout_id()));
                            }
                        }
                        if resp.double_clicked() && tool == crate::state::Tool::Hand {
                            let _ = app.run("view.fitOnScreen", json!({}));
                        }
                        if let Some((open_key, anchor)) = ctx.data(|d| d.get_temp::<(Id, Rect)>(flyout_id()))
                            && open_key == key
                        {
                            // `flyout` clears the state itself when a tool is picked or the press is outside it.
                            flyout(app, &ctx, &t, key, slot, anchor);
                        }
                    }
                });

            let mut end_ui = ui.new_child(UiBuilder::new().max_rect(right).layout(Layout::left_to_right(Align::Center)));
            end_ui.spacing_mut().item_spacing.x = gap;
            color_chip_button(app, &mut end_ui, &ctx);
            let panels_open = matches!(current_sheet(&ctx), Some(Sheet::Panel(_)));
            if icons::button(&mut end_ui, "layers", BTN, panels_open, "Panels").clicked() {
                set_sheet(&ctx, if panels_open { None } else { Some(Sheet::Panel(Group::Layers)) });
            }
        });
}

/// The tool flyout: the slot's tools as big rows above the tapped button.
fn flyout(app: &mut PhotocraftApp, ctx: &egui::Context, t: &Tokens, key: Id, slot: &[crate::state::Tool], anchor: Rect) {
    let screen = ctx.content_rect();
    let fw = 260.0f32.min((screen.width() - 16.0).max(120.0));
    let cx = anchor.center().x.clamp(screen.left() + fw / 2.0 + 8.0, (screen.right() - fw / 2.0 - 8.0).max(screen.left() + fw / 2.0 + 8.0));
    let mut picked = false;
    let area = egui::Area::new(key.with("flyout"))
        .order(egui::Order::Foreground)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(pos2(cx, anchor.top() - 8.0))
        .show(ctx, |ui| {
            Frame::popup(ui.style()).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for &item in slot {
                    let (r, ir) = ui.allocate_exact_size(vec2(fw, ROW_H), Sense::click());
                    if item == app.ui.tool {
                        ui.painter().rect_filled(r, t.radius_sm, t.accent_soft);
                    } else if ir.is_pointer_button_down_on() {
                        ui.painter().rect_filled(r, t.radius_sm, t.hover);
                    }
                    icons::paint(ui, Rect::from_center_size(pos2(r.left() + 26.0, r.center().y), vec2(24.0, 24.0)), icons::tool_icon(item), 22.0, t.icon);
                    ui.painter().text(pos2(r.left() + 54.0, r.center().y), Align2::LEFT_CENTER, tl!(item.label()), egui::FontId::proportional(16.0), t.text);
                    if ir.clicked() {
                        app.ui.tool = item;
                        picked = true;
                    }
                }
            });
        });
    let outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.press_origin().is_some_and(|p| !area.response.rect.contains(p) && !anchor.contains(p)));
    if picked || outside {
        ctx.data_mut(|d| d.remove::<(Id, Rect)>(flyout_id()));
    }
}

/// Foreground over background colour; tapping it opens the Colour panel.
fn color_chip_button(app: &mut PhotocraftApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(BTN, BTN), Sense::click());
    let on = current_sheet(ctx) == Some(Sheet::Panel(Group::Color));
    if on {
        ui.painter().rect_filled(rect, t.radius_sm, t.accent_soft);
    } else if resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(rect, t.radius_sm, t.hover);
    }
    let chip = 26.0;
    let fg = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(chip, chip));
    let bg = Rect::from_min_size(rect.min + vec2(BTN - chip - 6.0, BTN - chip - 6.0), vec2(chip, chip));
    let frame = |r: Rect| {
        ui.painter().rect_stroke(r, 3.0, Stroke::new(1.0, t.text.gamma_multiply(0.9)), egui::StrokeKind::Inside);
        ui.painter().rect_stroke(r, 3.0, Stroke::new(1.0, t.field_border), egui::StrokeKind::Outside);
    };
    ui.painter().rect_filled(bg, 3.0, color32(app.session.tools.background));
    frame(bg);
    ui.painter().rect_filled(fg.expand(2.0), 5.0, t.chrome);
    ui.painter().rect_filled(fg, 3.0, color32(app.session.tools.foreground));
    frame(fg);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Color")));
    if resp.clicked() {
        set_sheet(ctx, if on { None } else { Some(Sheet::Panel(Group::Color)) });
    }
}

// ----------------------------------------------------------------------------- sheet

const GROUPS: [(Group, &str, &str); 6] = [
    (Group::Layers, "layers", "Layers"),
    (Group::Color, "palette", "Color & Swatches"),
    (Group::Properties, "sliders-horizontal", "Properties"),
    (Group::History, "clock", "History"),
    (Group::Navigator, "navigation", "Navigator"),
    (Group::Character, "type", "Character"),
];

fn sheet(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let Some(kind) = current_sheet(&ctx) else { return };
    let t = Tokens::get(&ctx);
    let screen = ctx.content_rect();
    let wide = screen.width() >= WIDE;
    let frame = Frame::NONE.fill(t.dock).inner_margin(Margin::same(6));
    let max_h = (screen.height() * 0.85).max(200.0);
    let panel = match (wide, kind) {
        (true, _) => egui::Panel::right("touch_sheet_side").resizable(false).exact_size(380.0),
        (false, Sheet::Menu) => egui::Panel::bottom("touch_sheet_menu").resizable(true).default_size(screen.height() * 0.7).size_range(200.0..=max_h),
        (false, Sheet::Panel(_)) => egui::Panel::bottom("touch_sheet_panels").resizable(true).default_size(screen.height() * 0.45).size_range(200.0..=max_h),
    };
    let mut close = false;
    let mut run: Option<String> = None;
    panel.frame(frame).show(ui, |ui| {
        // Header: what this is, and a close button. Exactly one button tall: a row that filled the
        // panel centred the title halfway down and left no room for the content below it.
        ui.allocate_ui_with_layout(vec2(ui.available_width(), BTN), crate::direction::row(), |ui| {
            let title = match kind {
                Sheet::Menu => tl!("Menu").to_string(),
                Sheet::Panel(g) => GROUPS.iter().find(|x| x.0 == g).map(|x| tl!(x.2).to_string()).unwrap_or_default(),
            };
            ui.label(RichText::new(title).size(17.0).color(t.text));
            ui.with_layout(crate::direction::trailing(), |ui| {
                if icons::button(ui, "x", BTN, false, "Close").clicked() {
                    close = true;
                }
            });
        });
        match kind {
            Sheet::Menu => {
                let mut path: Vec<String> = ctx.data(|d| d.get_temp::<Vec<String>>(menu_path_id())).unwrap_or_default();
                egui::ScrollArea::vertical().id_salt("touch-menu").auto_shrink([false, false]).show(ui, |ui| {
                    run = crate::menus::touch_menu(app, ui, &mut path);
                });
                ctx.data_mut(|d| d.insert_temp(menu_path_id(), path));
            }
            Sheet::Panel(g) => panel_sheet(app, ui, &t, g, &ctx),
        }
    });
    if let Some(id) = run {
        close = true;
        let id = crate::menus::alt_click(id, false);
        if let Err(e) = crate::menus::invoke(app, &ctx, &id, json!({})) {
            app.ui.status = e;
            app.ui.status_error = true;
        }
    }
    if close {
        set_sheet(&ctx, None);
    }
}

fn panel_sheet(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens, group: Group, ctx: &egui::Context) {
    // Which panel: a scrolling row of icons.
    egui::ScrollArea::horizontal().id_salt("touch-groups").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (g, icon, name) in GROUPS {
                if icons::button(ui, icon, BTN, g == group, name).clicked() {
                    set_sheet(ctx, Some(Sheet::Panel(g)));
                }
            }
        });
    });
    // Which tab of that panel (Layers | Channels | Paths, …).
    let tabs = group.tabs(t.pro);
    let current = (*group.tab_mut(&mut app.ui.dock_tabs)).min(tabs.len().saturating_sub(1));
    if tabs.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            for (i, name) in tabs.iter().enumerate() {
                if ui.add(egui::Button::selectable(i == current, tl!(*name)).min_size(vec2(0.0, 40.0))).clicked() {
                    *group.tab_mut(&mut app.ui.dock_tabs) = i;
                }
            }
        });
    }
    ui.add_space(4.0);
    // Layers and History lay out their own scrolling list and footer; the rest scroll here.
    if group.scrolls_itself(current) {
        crate::panels::dock_body(app, ui, group, current);
    } else {
        egui::ScrollArea::vertical().id_salt(("touch-panel", group.key(), current)).auto_shrink([false, false]).show(ui, |ui| {
            crate::panels::dock_body(app, ui, group, current);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_by_default_and_toggles() {
        let ctx = egui::Context::default();
        assert!(!on(&ctx));
        enable(&ctx, true);
        assert!(on(&ctx));
        enable(&ctx, false);
        assert!(!on(&ctx));
    }

    #[test]
    fn colour_chip_clamps_out_of_range_channels() {
        assert_eq!(color32([2.0, -1.0, 0.5, 1.0]), Color32::from_rgba_unmultiplied(255, 0, 128, 255));
    }

    #[test]
    fn touch_style_keeps_hit_targets_at_44_points() {
        let ctx = egui::Context::default();
        enable(&ctx, true);
        let s = ctx.global_style();
        assert!(s.spacing.interact_size.min_elem() >= 44.0);
    }
}

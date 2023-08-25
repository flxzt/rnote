// Imports
use super::RnCanvas;
use gtk4::{gdk, glib, prelude::*, Native};
use rnote_compose::penevent::ShortcutKey;
use rnote_compose::penevent::{KeyboardKey, PenState};
use rnote_compose::penevent::{ModifierKey, PenEvent};
use rnote_compose::penpath::Element;
use rnote_engine::ext::EventPropagationExt;
use rnote_engine::pens::penholder::BacklogPolicy;
use rnote_engine::pens::PenMode;
use rnote_engine::WidgetFlags;
use std::time::{Duration, Instant};

// Returns whether the event should be inhibited from propagating, and the new pen state
pub(crate) fn handle_pointer_controller_event(
    canvas: &RnCanvas,
    event: &gdk::Event,
    mut state: PenState,
) -> (glib::Propagation, PenState) {
    let now = Instant::now();
    let mut widget_flags = WidgetFlags::default();
    let touch_drawing = canvas.touch_drawing();
    let gdk_event_type = event.event_type();
    let gdk_modifiers = event.modifier_state();
    let _gdk_device = event.device().unwrap();
    let backlog_policy = canvas.engine_ref().penholder.backlog_policy();
    let is_stylus = event_is_stylus(event);

    //std::thread::sleep(std::time::Duration::from_millis(100));
    //super::input::debug_gdk_event(event);

    if reject_pointer_input(event, touch_drawing) {
        return (glib::Propagation::Proceed, state);
    }

    let mut handle_pen_event = false;
    let mut propagation = glib::Propagation::Proceed;

    match gdk_event_type {
        gdk::EventType::MotionNotify => {
            //log::debug!("MotionNotify - modifiers: {gdk_modifiers:?}, is_stylus: {is_stylus}");

            if is_stylus {
                handle_pen_event = true;

                // like in gtk4 'gesturestylus.c:120' stylus proximity is detected this way,
                // in case ProximityIn & ProximityOut is not reported.
                if gdk_modifiers.contains(gdk::ModifierType::BUTTON1_MASK) {
                    state = PenState::Down;
                } else {
                    state = PenState::Proximity;
                }
            } else {
                // only handle no pressed button, primary and secondary mouse buttons.
                if gdk_modifiers.is_empty()
                    || gdk_modifiers.contains(gdk::ModifierType::BUTTON1_MASK)
                    || gdk_modifiers.contains(gdk::ModifierType::BUTTON3_MASK)
                {
                    handle_pen_event = true;
                }
            }
        }
        gdk::EventType::ButtonPress => {
            let button_event = event.downcast_ref::<gdk::ButtonEvent>().unwrap();
            let gdk_button = button_event.button();
            let mut handle_shortcut_key = false;

            //log::debug!("ButtonPress - button: {gdk_button}, is_stylus: {is_stylus}");

            if is_stylus {
                if gdk_button == gdk::BUTTON_PRIMARY
                    || gdk_button == gdk::BUTTON_SECONDARY
                    || gdk_button == gdk::BUTTON_MIDDLE
                {
                    handle_pen_event = true;
                    handle_shortcut_key = true;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if gdk_button == gdk::BUTTON_PRIMARY || gdk_button == gdk::BUTTON_SECONDARY {
                    handle_pen_event = true;
                    handle_shortcut_key = true;
                    state = PenState::Down;
                }
            }

            if handle_shortcut_key {
                let shortcut_key = retrieve_button_shortcut_key(gdk_button, is_stylus);

                if let Some(shortcut_key) = shortcut_key {
                    let (ep, wf) = canvas
                        .engine_mut()
                        .handle_pressed_shortcut_key(shortcut_key, now);
                    widget_flags |= wf;
                    propagation = ep.into_glib();
                }
            }
        }
        gdk::EventType::ButtonRelease => {
            let button_event = event.downcast_ref::<gdk::ButtonEvent>().unwrap();
            let gdk_button = button_event.button();

            //log::debug!("ButtonRelease - button: {gdk_button}, is_stylus: {is_stylus}");

            if is_stylus {
                if gdk_button == gdk::BUTTON_PRIMARY
                    || gdk_button == gdk::BUTTON_SECONDARY
                    || gdk_button == gdk::BUTTON_MIDDLE
                {
                    handle_pen_event = true;
                }

                // again, this is the method to detect proximity on stylus.
                if gdk_button == gdk::BUTTON_PRIMARY {
                    state = PenState::Up;
                } else {
                    state = PenState::Proximity;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if gdk_button == gdk::BUTTON_PRIMARY || gdk_button == gdk::BUTTON_SECONDARY {
                    state = PenState::Up;
                    handle_pen_event = true;
                }
            };
        }
        gdk::EventType::ProximityIn => {
            state = PenState::Proximity;
            handle_pen_event = true;
        }
        gdk::EventType::ProximityOut => {
            state = PenState::Up;
            handle_pen_event = true;
        }
        // We early-returned when detecting touch input and touch-drawing is not enabled,
        // so it is fine to always handle it here.
        gdk::EventType::TouchBegin => {
            state = PenState::Down;
            handle_pen_event = true;
        }
        gdk::EventType::TouchUpdate => {
            state = PenState::Down;
            handle_pen_event = true;
        }
        gdk::EventType::TouchEnd => {
            state = PenState::Up;
            handle_pen_event = true;
        }
        gdk::EventType::TouchCancel => {
            state = PenState::Up;
            handle_pen_event = true;
        }
        _ => {}
    };

    if handle_pen_event {
        let Some(elements) = retrieve_pointer_elements(canvas, now, event, backlog_policy) else {
            return (glib::Propagation::Proceed, state);
        };
        let modifier_keys = retrieve_modifier_keys(event.modifier_state());
        let pen_mode = retrieve_pen_mode(event);

        for (element, event_time) in elements {
            //log::debug!("handle pen event, state: {state:?}, event_time_d: {:?}, modifier_keys: {modifier_keys:?}, pen_mode: {pen_mode:?}", now.duration_since(event_time));

            match state {
                PenState::Up => {
                    canvas.enable_drawing_cursor(false);

                    let (ep, wf) = canvas.engine_mut().handle_pen_event(
                        PenEvent::Up {
                            element,
                            modifier_keys: modifier_keys.clone(),
                        },
                        pen_mode,
                        event_time,
                    );
                    widget_flags |= wf;
                    propagation = ep.into_glib();
                }
                PenState::Proximity => {
                    canvas.enable_drawing_cursor(false);

                    let (ep, wf) = canvas.engine_mut().handle_pen_event(
                        PenEvent::Proximity {
                            element,
                            modifier_keys: modifier_keys.clone(),
                        },
                        pen_mode,
                        event_time,
                    );
                    widget_flags |= wf;
                    propagation = ep.into_glib();
                }
                PenState::Down => {
                    canvas.grab_focus();
                    canvas.enable_drawing_cursor(true);

                    let (ep, wf) = canvas.engine_mut().handle_pen_event(
                        PenEvent::Down {
                            element,
                            modifier_keys: modifier_keys.clone(),
                        },
                        pen_mode,
                        event_time,
                    );
                    widget_flags |= wf;
                    propagation = ep.into_glib();
                }
            }
        }
    }

    canvas.emit_handle_widget_flags(widget_flags);
    (propagation, state)
}

pub(crate) fn handle_key_controller_key_pressed(
    canvas: &RnCanvas,
    key: gdk::Key,
    modifier: gdk::ModifierType,
) -> glib::Propagation {
    //log::debug!("key pressed - key: {:?}, raw: {:?}, modifier: {:?}", key, raw, modifier);
    canvas.grab_focus();

    let now = Instant::now();
    let keyboard_key = retrieve_keyboard_key(key);
    let modifier_keys = retrieve_modifier_keys(modifier);

    let (propagation, widget_flags) = canvas.engine_mut().handle_pen_event(
        PenEvent::KeyPressed {
            keyboard_key,
            modifier_keys,
        },
        None,
        now,
    );
    canvas.emit_handle_widget_flags(widget_flags);

    propagation.into_glib()
}

pub(crate) fn handle_imcontext_text_commit(canvas: &RnCanvas, text: &str) {
    let now = Instant::now();

    let (_ep, widget_flags) = canvas.engine_mut().handle_pen_event(
        PenEvent::Text {
            text: text.to_string(),
        },
        None,
        now,
    );
    canvas.emit_handle_widget_flags(widget_flags);
}

#[allow(unused)]
fn debug_gdk_event(event: &gdk::Event) {
    let pos = event
        .position()
        .map(|(x, y)| format!("x: {x:.1}, y: {y:.1}"));
    log::debug!(
        "(pos: {:?}, device: {:?}, modifier: {:?}, event_type: {:?}, tool type: {:?}, input source: {:?}",
        pos,
        event.device(),
        event.modifier_state(),
        event.event_type(),
        event.device_tool().map(|t| t.tool_type()),
        event.device().map(|d| d.source())
    );
}

/// Returns true if input should be rejected
fn reject_pointer_input(event: &gdk::Event, touch_drawing: bool) -> bool {
    if touch_drawing {
        if event.device().unwrap().num_touches() > 1 {
            return true;
        }
    } else {
        let event_type = event.event_type();
        if event.is_pointer_emulated()
            || event_type == gdk::EventType::TouchBegin
            || event_type == gdk::EventType::TouchUpdate
            || event_type == gdk::EventType::TouchEnd
            || event_type == gdk::EventType::TouchCancel
        {
            return true;
        }
    }
    false
}

fn event_is_stylus(event: &gdk::Event) -> bool {
    // As in gtk4 'gtkgesturestylus.c:106' we detect if the pointer is a stylus when it has a device tool
    event.device_tool().is_some()
}

fn retrieve_pointer_elements(
    canvas: &RnCanvas,
    now: Instant,
    event: &gdk::Event,
    backlog_policy: BacklogPolicy,
) -> Option<Vec<(Element, Instant)>> {
    // Retrieve the transform directly from the event, just like in `gtkgesturestylus.c`'s `get_backlog()`
    let event_native = Native::for_surface(&event.surface()?)?;
    let (surface_trans_x, surface_trans_y) = event_native.surface_transform();
    // retrieving the pressure only works when the event has a device tool (== is a stylus),
    // else we get SIGSEGV when trying to access (TODO: report this to gtk-rs)
    let is_stylus = event_is_stylus(event);
    let event_time = event.time();

    let mut elements = Vec::with_capacity(1);

    // Transforms the pos given in surface coordinate space to the canvas document coordinate space
    let transform_pos = |pos: na::Vector2<f64>| -> na::Vector2<f64> {
        event_native
            .translate_coordinates(canvas, pos[0] - surface_trans_x, pos[1] - surface_trans_y)
            .map(|(x, y)| {
                (canvas.engine_ref().camera.transform().inverse() * na::point![x, y]).coords
            })
            .unwrap()
    };

    if event.event_type() == gdk::EventType::MotionNotify
        && backlog_policy != BacklogPolicy::DisableBacklog
    {
        let mut prev_delta = Duration::ZERO;

        let mut entries = vec![];
        for entry in event.history().into_iter().rev() {
            let available_axes = entry.flags();
            if !(available_axes.contains(gdk::AxisFlags::X)
                && available_axes.contains(gdk::AxisFlags::Y))
            {
                continue;
            }

            let entry_delta = Duration::from_millis(event_time.saturating_sub(entry.time()) as u64);
            let Some(entry_time) = now.checked_sub(entry_delta) else {
                continue;
            };

            if let BacklogPolicy::Limit(delta_limit) = backlog_policy {
                // We go back in time, so `entry_delta` will increase
                //
                // If the backlog input rate is higher than the limit, filter it out
                if entry_delta.saturating_sub(prev_delta) < delta_limit {
                    continue;
                }
            }
            prev_delta = entry_delta;

            let axes = entry.axes();
            let pos = transform_pos(na::vector![
                axes[crate::utils::axis_use_idx(gdk::AxisUse::X)],
                axes[crate::utils::axis_use_idx(gdk::AxisUse::Y)]
            ]);
            let pressure = if is_stylus {
                axes[crate::utils::axis_use_idx(gdk::AxisUse::Pressure)]
            } else {
                Element::PRESSURE_DEFAULT
            };

            entries.push((Element::new(pos, pressure), entry_time));
        }

        elements.extend(entries.into_iter().rev());
    }

    let pos = event
        .position()
        .map(|(x, y)| transform_pos(na::vector![x, y]))?;

    let pressure = if is_stylus {
        event.axis(gdk::AxisUse::Pressure).unwrap()
    } else {
        Element::PRESSURE_DEFAULT
    };

    elements.push((Element::new(pos, pressure), now));

    Some(elements)
}

pub(crate) fn retrieve_button_shortcut_key(
    gdk_button: u32,
    is_stylus: bool,
) -> Option<ShortcutKey> {
    match (is_stylus, gdk_button) {
        (_, gdk::BUTTON_PRIMARY) => None,
        (false, gdk::BUTTON_SECONDARY) => Some(ShortcutKey::MouseSecondaryButton),
        (true, gdk::BUTTON_SECONDARY) => Some(ShortcutKey::StylusPrimaryButton),
        (true, gdk::BUTTON_MIDDLE) => Some(ShortcutKey::StylusSecondaryButton),
        (_, _) => None,
    }
}

pub(crate) fn retrieve_modifier_keys(modifier: gdk::ModifierType) -> Vec<ModifierKey> {
    let mut keys = vec![];

    if modifier.contains(gdk::ModifierType::SHIFT_MASK) {
        keys.push(ModifierKey::KeyboardShift);
    }
    if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
        keys.push(ModifierKey::KeyboardCtrl);
    }
    if modifier.contains(gdk::ModifierType::ALT_MASK) {
        keys.push(ModifierKey::KeyboardAlt);
    }
    keys
}

fn retrieve_pen_mode(event: &gdk::Event) -> Option<PenMode> {
    let device_tool = event.device_tool()?;
    match device_tool.tool_type() {
        gdk::DeviceToolType::Pen => Some(PenMode::Pen),
        gdk::DeviceToolType::Eraser => Some(PenMode::Eraser),
        _ => None,
    }
}

pub(crate) fn retrieve_keyboard_key(gdk_key: gdk::Key) -> KeyboardKey {
    if let Some(keychar) = gdk_key.to_unicode() {
        KeyboardKey::Unicode(keychar).filter_convert_unicode_control_chars()
    } else {
        match gdk_key {
            gdk::Key::BackSpace => KeyboardKey::BackSpace,
            gdk::Key::Tab => KeyboardKey::HorizontalTab,
            gdk::Key::Linefeed => KeyboardKey::Linefeed,
            gdk::Key::Return => KeyboardKey::CarriageReturn,
            gdk::Key::Escape => KeyboardKey::Escape,
            gdk::Key::Delete => KeyboardKey::Delete,
            gdk::Key::Down => KeyboardKey::NavDown,
            gdk::Key::Up => KeyboardKey::NavUp,
            gdk::Key::Left => KeyboardKey::NavLeft,
            gdk::Key::Right => KeyboardKey::NavRight,
            gdk::Key::Shift_L => KeyboardKey::ShiftLeft,
            gdk::Key::Shift_R => KeyboardKey::ShiftRight,
            gdk::Key::Control_L => KeyboardKey::CtrlLeft,
            gdk::Key::Control_R => KeyboardKey::CtrlRight,
            gdk::Key::Home => KeyboardKey::Home,
            gdk::Key::End => KeyboardKey::End,
            _ => KeyboardKey::Unsupported,
        }
    }
}

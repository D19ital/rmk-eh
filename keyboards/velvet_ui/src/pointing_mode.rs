use rmk::event::{
    publish_event, KeyPos, KeyboardEvent, KeyboardEventPos, LayerChangeEvent,
    PointingProcessorEvent,
};
use rmk::input_device::pointing::{
    CaretConfig, CursorConfig, PointingMode, ScrollConfig, SniperConfig,
};
use rmk::macros::processor;
use rmk::types::keycode::HidKeyCode;

const TRACKBALL_DEVICE_ID: u8 = 0;

const LAYER_MOUSE: u8 = 4;
const MODE_SNIPER_USER: u8 = 10;
const MODE_SCROLL_USER: u8 = 11;
const MODE_CARET_USER: u8 = 12;

#[processor(subscribe = [KeyboardEvent, LayerChangeEvent])]
pub struct VelvetPointingModeProcessor {
    active_layer: u8,
    mode_active: bool,
}

impl VelvetPointingModeProcessor {
    pub fn new() -> Self {
        Self {
            active_layer: 0,
            mode_active: false,
        }
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        self.active_layer = event.0;
        if self.active_layer != LAYER_MOUSE && self.mode_active {
            rmk::vial_settings::set_mode(0);
            self.mode_active = false;
            publish_current_vial_mode();
        }
    }

    async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        if self.active_layer != LAYER_MOUSE {
            return;
        }

        let KeyboardEventPos::Key(pos) = event.pos else {
            return;
        };

        let Some(user_id) = pointing_mode_user_for_pos(pos) else {
            return;
        };

        if event.pressed {
            let mode = match user_id {
                MODE_SNIPER_USER => 1,
                MODE_SCROLL_USER => 2,
                MODE_CARET_USER => 3,
                _ => return,
            };
            rmk::vial_settings::set_mode(mode);
            self.mode_active = true;
            publish_current_vial_mode();
        } else if !rmk::vial_settings::sticky_mode() {
            rmk::vial_settings::set_mode(0);
            self.mode_active = false;
            publish_current_vial_mode();
        }
    }
}

fn pointing_mode_user_for_pos(pos: KeyPos) -> Option<u8> {
    match (pos.row, pos.col) {
        (1, 5) | (5, 0) => Some(MODE_SNIPER_USER),
        (1, 1) | (5, 4) => Some(MODE_SCROLL_USER),
        (2, 5) | (6, 0) => Some(MODE_CARET_USER),
        _ => None,
    }
}

fn current_vial_pointing_mode() -> PointingMode {
    match rmk::vial_settings::mode() {
        1 => PointingMode::Sniper(SniperConfig {
            multiplier: 1,
            divisor: rmk::vial_settings::sniper_sens().clamp(1, u8::MAX as i16) as u8,
            invert_x: false,
            invert_y: false,
        }),
        2 => PointingMode::Scroll(ScrollConfig {
            multiplier_x: 1,
            divisor_x: rmk::vial_settings::scroll_sens().clamp(1, u8::MAX as i32) as u8,
            multiplier_y: 1,
            divisor_y: rmk::vial_settings::scroll_sens().clamp(1, u8::MAX as i32) as u8,
            invert_x: false,
            invert_y: rmk::vial_settings::invert_scroll(),
        }),
        3 => PointingMode::Caret(CaretConfig {
            disable_x: false,
            disable_y: false,
            invert_x: false,
            invert_y: rmk::vial_settings::invert_text(),
            threshold: rmk::vial_settings::text_sens().clamp(1, i16::MAX as i32) as i16,
            keycode_up: HidKeyCode::Up,
            keycode_down: HidKeyCode::Down,
            keycode_left: HidKeyCode::Left,
            keycode_right: HidKeyCode::Right,
        }),
        _ => PointingMode::Cursor(CursorConfig::default()),
    }
}

fn publish_current_vial_mode() {
    publish_event(PointingProcessorEvent {
        device_id: TRACKBALL_DEVICE_ID,
        mode: current_vial_pointing_mode(),
    });
}

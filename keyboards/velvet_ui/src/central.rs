#![no_main]
#![no_std]

mod pointing_mode;

use rmk::macros::rmk_central;

#[rmk_central]
mod keyboard_central {
    #[register_processor(event)]
    fn velvet_pointing_mode_processor() -> crate::pointing_mode::VelvetPointingModeProcessor {
        crate::pointing_mode::VelvetPointingModeProcessor::new()
    }
}

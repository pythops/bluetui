use crate::requests::{
    confirmation::Confirmation, display_passkey::DisplayPasskey, display_pin_code::DisplayPinCode,
    enter_passkey::EnterPasskey, enter_pin_code::EnterPinCode,
};

pub mod confirmation;
pub mod display_passkey;
pub mod display_pin_code;
pub mod enter_passkey;
pub mod enter_pin_code;

#[derive(Debug, Default)]
pub struct Requests {
    pub confirmation: Option<Confirmation>,
    pub enter_pin_code: Option<EnterPinCode>,
    pub enter_passkey: Option<EnterPasskey>,
    pub display_pin_code: Option<DisplayPinCode>,
    pub display_passkey: Option<DisplayPasskey>,
}

impl Requests {
    pub fn init_confirmation(&mut self, req: Confirmation) {
        self.confirmation = Some(req);
    }
    pub fn init_enter_pin_code(&mut self, req: EnterPinCode) {
        self.enter_pin_code = Some(req);
    }
    pub fn init_enter_passkey(&mut self, req: EnterPasskey) {
        self.enter_passkey = Some(req);
    }
    pub fn init_display_pin_code(&mut self, req: DisplayPinCode) {
        self.display_pin_code = Some(req);
    }
    pub fn init_display_passkey(&mut self, req: DisplayPasskey) {
        self.display_passkey = Some(req);
    }
}

fn pad_string(input: &str, length: usize) -> String {
    let current_length = input.chars().count();
    if current_length >= length {
        input.to_string()
    } else {
        format!("{:<width$}", input, width = length)
    }
}

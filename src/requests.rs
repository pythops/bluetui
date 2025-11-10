use crate::requests::{
    confirmation::Confirmation, enter_passkey::EnterPasskey, enter_pin_code::EnterPinCode,
};

pub mod confirmation;
pub mod enter_passkey;
pub mod enter_pin_code;

#[derive(Debug, Default)]
pub struct Requests {
    pub confirmation: Option<Confirmation>,
    pub enter_pin_code: Option<EnterPinCode>,
    pub enter_passkey: Option<EnterPasskey>,
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
}

fn pad_string(input: &str, length: usize) -> String {
    let current_length = input.chars().count();
    if current_length >= length {
        input.to_string()
    } else {
        format!("{:<width$}", input, width = length)
    }
}

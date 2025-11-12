use async_channel::{Receiver, Sender};
use tokio::sync::mpsc::UnboundedSender;

use bluer::agent::{
    DisplayPasskey, DisplayPinCode, ReqError, ReqResult, RequestConfirmation, RequestPasskey,
    RequestPinCode,
};

use crate::{
    event::Event,
    requests::{
        confirmation::Confirmation, enter_passkey::EnterPasskey, enter_pin_code::EnterPinCode,
    },
};

#[derive(Debug, Clone)]
pub struct AuthAgent {
    pub event_sender: UnboundedSender<Event>,
    pub tx_cancel: Sender<()>,
    pub rx_cancel: Receiver<()>,
    pub tx_pin_code: Sender<String>,
    pub rx_pin_code: Receiver<String>,
    pub tx_display_pin_code: Sender<()>,
    pub rx_display_pin_code: Receiver<()>,
    pub tx_display_passkey: Sender<()>,
    pub rx_display_passkey: Receiver<()>,
    pub tx_passkey: Sender<u32>,
    pub rx_passkey: Receiver<u32>,
    pub tx_request_confirmation: Sender<bool>,
    pub rx_request_confirmation: Receiver<bool>,
}

impl AuthAgent {
    pub fn new(sender: UnboundedSender<Event>) -> Self {
        let (tx_passkey, rx_passkey) = async_channel::unbounded();
        let (tx_display_passkey, rx_display_passkey) = async_channel::unbounded();

        let (tx_pin_code, rx_pin_code) = async_channel::unbounded();
        let (tx_display_pin_code, rx_display_pin_code) = async_channel::unbounded();

        let (tx_request_confirmation, rx_request_confirmation) = async_channel::unbounded();
        let (tx_cancel, rx_cancel) = async_channel::unbounded();

        Self {
            event_sender: sender,
            tx_cancel,
            rx_cancel,
            tx_pin_code,
            rx_pin_code,
            tx_display_pin_code,
            rx_display_pin_code,
            tx_display_passkey,
            rx_display_passkey,
            tx_passkey,
            rx_passkey,
            tx_request_confirmation,
            rx_request_confirmation,
        }
    }
}

pub async fn request_confirmation(request: RequestConfirmation, agent: AuthAgent) -> ReqResult<()> {
    agent
        .event_sender
        .send(Event::RequestConfirmation(Confirmation::new(
            request.adapter,
            request.device,
            request.passkey,
        )))
        .unwrap();

    tokio::select! {
    r = agent.rx_request_confirmation.recv() =>  {
                match r {
                    Ok(v) => {
                        match v  {
                            true => Ok(()),
                            false =>  Err(ReqError::Rejected)
                        }
                    }
                    Err(_) => {
                        Err(ReqError::Canceled)
                    }
            }

        }

    _ = agent.rx_cancel.recv() => {
                Err(ReqError::Canceled)
        }

    }
}

pub async fn request_pin_code(request: RequestPinCode, agent: AuthAgent) -> ReqResult<String> {
    agent
        .event_sender
        .send(Event::RequestEnterPinCode(EnterPinCode::new(
            request.adapter,
            request.device,
        )))
        .unwrap();

    tokio::select! {
    r = agent.rx_pin_code.recv() =>  {
                match r {
                    Ok(v) => Ok(v),
                    Err(_) => Err(ReqError::Canceled)
                }

        }

    _ = agent.rx_cancel.recv() => {
                Err(ReqError::Canceled)
        }

    }
}

pub async fn request_passkey(request: RequestPasskey, agent: AuthAgent) -> ReqResult<u32> {
    agent
        .event_sender
        .send(Event::RequestEnterPasskey(EnterPasskey::new(
            request.adapter,
            request.device,
        )))
        .unwrap();

    tokio::select! {
    r = agent.rx_passkey.recv() =>  {
            match r {
                Ok(v) => Ok(v),
                Err(_) => Err(ReqError::Canceled)
            }
        }

    _ = agent.rx_cancel.recv() => {
                Err(ReqError::Canceled)
        }

    }
}

pub async fn display_pin_code(request: DisplayPinCode, agent: AuthAgent) -> ReqResult<()> {
    agent
        .event_sender
        .send(Event::RequestDisplayPinCode(
            crate::requests::display_pin_code::DisplayPinCode::new(
                request.adapter,
                request.device,
                request.pincode,
            ),
        ))
        .unwrap();

    tokio::select! {
    _ = agent.rx_display_pin_code.recv() => {
            Ok(())
        }

    _ = agent.rx_cancel.recv() => {
            Err(ReqError::Canceled)
        }
    }
}

pub async fn display_passkey(request: DisplayPasskey, agent: AuthAgent) -> ReqResult<()> {
    let _ = agent.event_sender.send(Event::RequestDisplayPasskey(
        crate::requests::display_passkey::DisplayPasskey::new(
            request.adapter,
            request.device,
            request.passkey,
            request.entered,
        ),
    ));

    tokio::select! {
    _ = agent.rx_display_passkey.recv() => {
            Ok(())
        }

    _ = agent.rx_cancel.recv() => {
            Err(ReqError::Canceled)
        }
    }
}

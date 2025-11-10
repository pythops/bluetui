use async_channel::{Receiver, Sender};
use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::mpsc::UnboundedSender;

use bluer::agent::{ReqError, ReqResult, RequestConfirmation, RequestPasskey, RequestPinCode};

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
    pub tx_passkey: Sender<u32>,
    pub rx_passkey: Receiver<u32>,
    pub tx_request_confirmation: Sender<bool>,
    pub rx_request_confirmation: Receiver<bool>,
    pub request_passkey: Arc<AtomicBool>,
    pub request_pin_code: Arc<AtomicBool>,
    pub request_confirmation: Arc<AtomicBool>,
    pub request_authorization: Arc<AtomicBool>,
    pub request_display_pin: Arc<AtomicBool>,
    pub request_display_passkey: Arc<AtomicBool>,
}

impl AuthAgent {
    pub fn new(sender: UnboundedSender<Event>) -> Self {
        let (tx_passkey, rx_passkey) = async_channel::unbounded();
        let (tx_pin_code, rx_pin_code) = async_channel::unbounded();
        let (tx_request_confirmation, rx_request_confirmation) = async_channel::unbounded();
        let (tx_cancel, rx_cancel) = async_channel::unbounded();

        Self {
            event_sender: sender,
            tx_cancel,
            rx_cancel,
            tx_pin_code,
            rx_pin_code,
            tx_passkey,
            rx_passkey,
            tx_request_confirmation,
            rx_request_confirmation,
            request_passkey: Arc::new(AtomicBool::new(false)),
            request_pin_code: Arc::new(AtomicBool::new(false)),
            request_confirmation: Arc::new(AtomicBool::new(false)),
            request_authorization: Arc::new(AtomicBool::new(false)),
            request_display_pin: Arc::new(AtomicBool::new(false)),
            request_display_passkey: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub async fn request_confirmation(request: RequestConfirmation, agent: AuthAgent) -> ReqResult<()> {
    agent
        .request_confirmation
        .store(true, std::sync::atomic::Ordering::Relaxed);

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
        .request_pin_code
        .store(true, std::sync::atomic::Ordering::Relaxed);

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
        .request_passkey
        .store(true, std::sync::atomic::Ordering::Relaxed);

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

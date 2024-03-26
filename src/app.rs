use anyhow::{anyhow, Result};
use bluer::gatt::remote::Characteristic;
use bluer::{Address, Device};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::ancs::notification::ANCSNotification;
use crate::ancs::notification_source::{NotificationEvent, NOTIFICATION_SOURCE_UUID};
use crate::ancs::ANCS_SERVICE_UUID;
use crate::utils::find_characteristic;

#[derive(Debug)]
enum AppState {
    Init,
    FoundDevice,
    RunningANCS,
}

#[derive(Debug)]
pub struct App {
    state: AppState,
    address: Address,
    iphone: Option<Device>,
    notification_source_char: Option<Characteristic>,
    notification_event_tx: Sender<NotificationEvent>,
    notification_event_rx: Receiver<NotificationEvent>,
    notifications: HashMap<u32, ANCSNotification>,
}

impl App {
    pub async fn new(config: config::Config) -> Self {
        let (tx, rx) = mpsc::channel(64);

        Self {
            state: AppState::Init,
            address: bluer::Address::from_str(&config.get::<String>("address").unwrap()).unwrap(),
            iphone: None,
            notification_source_char: None,
            notification_event_tx: tx,
            notification_event_rx: rx,
            notifications: HashMap::new(),
        }
    }

    pub async fn find_device() -> Result<bluer::Device> {
        Err(anyhow!("Cannot find iPhone"))
    }

    pub async fn run(&mut self) {
        loop {
            match self.state {
                AppState::Init => {
                    self.iphone = bluer::Session::new()
                        .await
                        .unwrap()
                        .default_adapter()
                        .await
                        .unwrap()
                        .device(self.address)
                        .ok();
                    if self.iphone.is_some() {
                        self.state = AppState::FoundDevice;
                    }
                }
                AppState::FoundDevice => {
                    let notification_source_char = find_characteristic(
                        &self.iphone.as_mut().unwrap(),
                        ANCS_SERVICE_UUID,
                        NOTIFICATION_SOURCE_UUID,
                    )
                    .await;
                    if notification_source_char.is_some() {
                        tokio::spawn(crate::ancs::notification_source::listener(
                            notification_source_char.unwrap(),
                            self.notification_event_tx,
                        ));
                        self.state = AppState::RunningANCS;
                    }
                }
                AppState::RunningANCS => continue,
            }

            if let Some(ns_event) = self.notification_event_rx.recv().await {
                if self.notifications.get(&ns_event.notification_id).is_none() {
                    self.notifications
                        .insert(
                            ns_event.notification_id,
                            ANCSNotification::new(ns_event.clone()),
                        )
                        .unwrap();
                }
                println!("{:#?}", ns_event);
            }
        }
    }
}

use bluer::{gatt::remote::Characteristic, Uuid};
use futures::{pin_mut, StreamExt};
use tokio::sync::mpsc;

use std::fmt;

pub const NOTIFICATION_SOURCE_UUID: Uuid = Uuid::from_u128(0x9FBF120D630142D98C5825E699A21DBD);

// Contains notification event
#[derive(Clone)]
pub struct NotificationEvent {
    pub event_id: u8,
    pub event_flags: u8,
    pub category_id: u8,
    pub category_count: u8,
    pub notification_id: u32,
}

impl NotificationEvent {
    pub fn from_buffer(buffer: Vec<u8>) -> Self {
        Self {
            event_id: buffer[0],
            event_flags: buffer[1],
            category_id: buffer[2],
            category_count: buffer[3],
            notification_id: (buffer[4] as u32)
                | ((buffer[5] as u32) << 8)
                | ((buffer[6] as u32) << 16)
                | ((buffer[7] as u32) << 24),
        }
    }
}

impl fmt::Display for NotificationEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "event id: {} event flags: {} category id: {} category count: {} notification id: {}",
            self.event_id,
            self.event_flags,
            self.category_id,
            self.category_count,
            self.notification_id
        )
    }
}

pub async fn listener(
    notification_source_char: Characteristic,
    notification_event_tx: mpsc::Sender<NotificationEvent>,
) {
    let notify = notification_source_char.notify().await.unwrap();
    pin_mut!(notify);

    loop {
        match notify.next().await {
            Some(buffer) => {
                notification_event_tx
                    .send(NotificationEvent::from_buffer(buffer))
                    .await
                    .unwrap();
            }
            None => continue,
        }
    }
}

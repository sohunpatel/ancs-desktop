#![allow(unused)]

use bluer::{gatt::remote::Characteristic, Uuid};
use futures::{pin_mut, StreamExt};
use tokio::sync::mpsc;

// UUID for characteristic
pub const CONTROL_POINT_UUID: Uuid = Uuid::from_u128(0x69D1D8F345E149A898219BBDFDAAD9D9);

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum CategoryID {
    Other,
    IncomingCall,
    MissedCall,
    Voicemail,
    Social,
    Schedule,
    Email,
    News,
    HealthAndFitness,
    BusinessAndFinance,
    Location,
    Entertainment,
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum EventID {
    NotificationAdded,
    NotificationModified,
    NotificationRemoved,
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum CommandID {
    GetNotificationAttributes,
    GetAppAttributes,
    PerformNotificationAction,
}

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum NotificationAttributeID {
    AppIdentifier,
    Title,
    Subtitle,
    Message,
    MessageSize,
    Date,
    PositiveActionLabel,
    NegativeActionLabel,
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum ActionID {
    Positive,
    Negative,
}

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum AppAttributeID {
    Displayname,
}

#[derive(Clone, Debug)]
pub struct NotificationAttributeCmd {
    pub notification_id: u32,
    pub attributes: Vec<NotificationAttributeID>,
}

impl NotificationAttributeCmd {
    pub fn new(notification_id: u32, attributes: Vec<NotificationAttributeID>) -> Self {
        Self {
            notification_id,
            attributes,
        }
    }

    pub fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let bytes = self.notification_id.to_ne_bytes();
        for byte in bytes {
            buffer.push(byte);
        }
        for attribute in self.attributes.clone() {
            buffer.push(attribute.clone() as u8);
            if attribute == NotificationAttributeID::Title
                || attribute == NotificationAttributeID::Subtitle
                || attribute == NotificationAttributeID::Message
            {
                buffer.push(std::u8::MAX);
                buffer.push(std::u8::MAX);
            }
        }
        buffer
    }
}

#[derive(Clone, Debug)]
pub struct AppAttributeCmd {
    pub app_identifier: String,
    pub attributes: Vec<AppAttributeID>,
}

impl AppAttributeCmd {
    pub fn new(app_identifier: String, attributes: Vec<AppAttributeID>) -> Self {
        Self {
            app_identifier,
            attributes,
        }
    }

    pub fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let bytes = self.app_identifier.bytes();
        for byte in bytes {
            buffer.push(byte);
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn little_endian() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let i: u32 = rng.gen();
            let buffer = NotificationAttributeCmd::new(i, Vec::new()).to_buffer();
            assert_eq!(i.to_le_bytes(), buffer.as_slice());
        }
    }

    #[test]
    fn notification_attributes() {
        let buffer =
            NotificationAttributeCmd::new(2, vec![NotificationAttributeID::Title]).to_buffer();
        assert_eq!(buffer, vec![2, 0, 0, 0, 1, 255, 255]);
    }
}

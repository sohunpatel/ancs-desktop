use bluer::{gatt::remote::Characteristic, Uuid};
use futures::{pin_mut, StreamExt};
use tokio::sync::mpsc;

use std::fmt;
use std::str;

use crate::ancs::control_point::{AppAttributeID, CommandID, NotificationAttributeID};

// UUID for characteristic
pub const DATA_SOURCE_UUID: Uuid = Uuid::from_u128(0x22EAC6E924D64BB5BE44B36ACE7C7BFB);

// Notification Attributes
pub struct NotificationAttributes {
    pub notification_id: u32,
    pub app_identifier: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub message: Option<String>,
    pub message_size: Option<u16>,
    pub date: Option<String>,
    pub positive_action_label: Option<String>,
    pub negative_action_label: Option<String>,
}

impl NotificationAttributes {
    pub fn from_buffer(buffer: Vec<u8>) -> Self {
        // clone buffer into local variable
        let mut buffer = buffer.clone();

        // the first byte is the message type which specifies whether the message is a notification
        // attribute or a app attribute
        // TODO: move this into listener function
        buffer.remove(0);

        let notification_id = (buffer.remove(0) as u32)
            | ((buffer.remove(0) as u32) << 8)
            | ((buffer.remove(0) as u32) << 16)
            | ((buffer.remove(0) as u32) << 24);
        let mut app_identifier = None;
        let mut title = None;
        let mut subtitle = None;
        let mut message = None;
        let mut message_size = None;
        let mut date = None;
        let mut positive_action_label = None;
        let mut negative_action_label = None;
        while buffer.len() > 0 {
            let attribute_id = buffer.remove(0);
            let attribute_length = (buffer.remove(0) as usize) | ((buffer.remove(0) as usize) << 8);
            if attribute_id == NotificationAttributeID::AppIdentifier as u8 {
                app_identifier = str::from_utf8(buffer.drain(0..attribute_length).as_slice())
                    .ok()
                    .map(str::to_string);
            } else if attribute_id == NotificationAttributeID::Title as u8 {
                title = str::from_utf8(buffer.drain(0..attribute_length).as_slice())
                    .ok()
                    .map(str::to_string);
            } else if attribute_id == NotificationAttributeID::Subtitle as u8 {
                subtitle = str::from_utf8(buffer.drain(0..attribute_length).as_slice())
                    .ok()
                    .map(str::to_string);
            } else if attribute_id == NotificationAttributeID::Message as u8 {
                message = str::from_utf8(buffer.drain(0..attribute_length).as_slice())
                    .ok()
                    .map(str::to_string);
            } else if attribute_id == NotificationAttributeID::MessageSize as u8 {
                message_size = Some((buffer.remove(0) as u16) | ((buffer.remove(0) as u16) << 8));
            } else if attribute_id == NotificationAttributeID::Date as u8 {
                date = str::from_utf8(buffer.drain(0..attribute_length).as_slice())
                    .ok()
                    .map(str::to_string);
            } else if attribute_id == NotificationAttributeID::PositiveActionLabel as u8 {
                positive_action_label =
                    str::from_utf8(buffer.drain(0..attribute_length).as_slice())
                        .ok()
                        .map(str::to_string);
            } else if attribute_id == NotificationAttributeID::NegativeActionLabel as u8 {
                negative_action_label =
                    str::from_utf8(buffer.drain(0..attribute_length).as_slice())
                        .ok()
                        .map(str::to_string);
            }
        }
        Self {
            notification_id,
            app_identifier,
            title,
            subtitle,
            message,
            message_size,
            date,
            positive_action_label,
            negative_action_label,
        }
    }
}

// Implement formatted display for Notification Attributes
impl fmt::Display for NotificationAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!("id: {}", self.notification_id);
        if let Some(app_identifier) = &self.app_identifier.clone() {
            output += &format!(" app identifier: {}", app_identifier);
        }
        if let Some(title) = &self.title.clone() {
            output += &format!(" title: {}", title);
        }
        if let Some(subtitle) = &self.subtitle.clone() {
            output += &format!(" subtitle: {}", subtitle);
        }
        if let Some(message) = &self.message.clone() {
            output += &format!(" message: {}", message);
        }
        if let Some(message_size) = self.message_size {
            output += &format!(" message_size: {}", message_size);
        }
        if let Some(date) = self.date.clone() {
            output += &format!(" date: {}", date);
        }
        if let Some(positive_action_label) = self.positive_action_label.clone() {
            output += &format!(" positive action label: {}", positive_action_label);
        }
        if let Some(negative_action_label) = self.negative_action_label.clone() {
            output += &format!(" negative action lable: {}", negative_action_label);
        }
        write!(f, "{}", output)
    }
}

// App Attributes
pub struct AppAttributes {
    pub app_identifier: String,
    pub display_name: Option<String>,
}

impl AppAttributes {
    pub fn from_buffer(buffer: Vec<u8>) -> Self {
        let mut buffer = buffer.clone();
        buffer.remove(0);
        let mut app_identifier = String::new();
        if let Some(null_terminator) = buffer.iter().position(|&b| b == 0) {
            app_identifier = str::from_utf8(buffer.drain(..null_terminator).as_slice())
                .unwrap()
                .to_string();
        }
        let mut display_name = None;
        while buffer.len() > 0 {
            let attribute_id = buffer.remove(0);
            let _ = buffer.drain(0..2).as_slice();
            if attribute_id == AppAttributeID::Displayname as u8 {
                display_name = str::from_utf8(buffer.drain(..).as_slice())
                    .ok()
                    .map(str::to_string);
            }
        }
        Self {
            app_identifier,
            display_name,
        }
    }
}

impl fmt::Display for AppAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!("app identifier: {}", self.app_identifier);
        if let Some(display_name) = self.display_name.clone() {
            output += &format!("{}", display_name);
        }
        write!(f, "{}", output)
    }
}

// asynchronous listener
pub async fn listener(
    data_source_char: Characteristic,
    notification_attributes_tx: mpsc::Sender<NotificationAttributes>,
    app_attributes_tx: mpsc::Sender<AppAttributes>,
) {
    let data = data_source_char.notify().await.unwrap();
    pin_mut!(data);

    loop {
        match data.next().await {
            Some(buffer) => {
                if buffer[0] == CommandID::GetNotificationAttributes as u8 {
                    notification_attributes_tx
                        .send(NotificationAttributes::from_buffer(buffer))
                        .await
                        .unwrap();
                } else if buffer[0] == CommandID::GetAppAttributes as u8 {
                    app_attributes_tx
                        .send(AppAttributes::from_buffer(buffer))
                        .await
                        .unwrap();
                }
            }
            None => continue,
        }
    }
}

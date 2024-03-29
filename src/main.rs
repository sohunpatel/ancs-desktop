mod ancs;
mod utils;

use config::Config;
use std::{collections::HashMap, str::FromStr};
use tokio::sync::mpsc;
use utils::find_characteristic;

use ancs::control_point::EventID;
use ancs::{
    control_point::CONTROL_POINT_UUID, data_source::DATA_SOURCE_UUID,
    notification::ANCSNotification, notification_source::NOTIFICATION_SOURCE_UUID,
    ANCS_SERVICE_UUID,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // use xdg spec to load config file and set log file location
    let xdg_dirs = xdg::BaseDirectories::with_prefix("ancs").unwrap();
    let config_path = xdg_dirs
        .find_config_file("ancs.toml")
        .expect("Could not find config file!");
    // Load config
    let config = Config::builder()
        .add_source(config::File::with_name(config_path.to_str().unwrap()))
        .build()
        .unwrap();

    // Get iphone device using MAC address from config
    let iphone = bluer::Session::new()
        .await
        .unwrap()
        .default_adapter()
        .await
        .unwrap()
        .device(bluer::Address::from_str(&config.get::<String>("address").unwrap()).unwrap())
        .unwrap();

    let notification_source_char =
        find_characteristic(&iphone, ANCS_SERVICE_UUID, NOTIFICATION_SOURCE_UUID)
            .await
            .unwrap();
    let _control_point_char = find_characteristic(&iphone, ANCS_SERVICE_UUID, CONTROL_POINT_UUID)
        .await
        .unwrap();
    let data_source_char = find_characteristic(&iphone, ANCS_SERVICE_UUID, DATA_SOURCE_UUID)
        .await
        .unwrap();

    let (notification_event_tx, mut notification_event_rx) = mpsc::channel(64);
    let (notification_attributes_tx, mut notification_attributes_rx) = mpsc::channel(64);
    let (app_attributes_tx, mut app_attributes_rx) = mpsc::channel(64);

    tokio::spawn(ancs::notification_source::listener(
        notification_source_char,
        notification_event_tx,
    ));

    tokio::spawn(ancs::data_source::listener(
        data_source_char,
        notification_attributes_tx,
        app_attributes_tx,
    ));

    libnotify::init("ancs").unwrap();

    let mut notifications: HashMap<u32, ANCSNotification> = HashMap::new();
    let mut display_names: HashMap<String, String> = HashMap::new();

    // Main thread code
    loop {
        if let Some(event) = notification_event_rx.recv().await {
            if event.event_id == EventID::NotificationAdded as u8 {
                notifications.insert(event.notification_id, ANCSNotification::new());
            } else if event.event_id == EventID::NotificationRemoved as u8 {
                notifications.remove(&event.notification_id).unwrap();
            }
            println!(
                "{} {} {} {} {}",
                event.event_id,
                event.event_flags,
                event.category_id,
                event.category_count,
                event.notification_id
            );
        }
        if let Some(attributes) = notification_attributes_rx.recv().await {
            let notification = notifications.get_mut(&attributes.notification_id).unwrap();
            notification.update(attributes.title, attributes.message);
        }
        if let Some(attributes) = app_attributes_rx.recv().await {
            if let Some(display_name) = attributes.display_name {
                display_names.insert(attributes.app_identifier, display_name).unwrap();
            }
        }
    }
}

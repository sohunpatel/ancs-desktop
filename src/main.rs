#![allow(warnings)]

mod ancs;
mod utils;

use bluer::DiscoveryFilter;
use config::{Config, ConfigBuilder};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use tokio::sync::mpsc;
use utils::find_characteristic;

use ancs::{
    control_point::CONTROL_POINT_UUID, data_source::DATA_SOURCE_UUID,
    notification::ANCSNotification, notification_source::NOTIFICATION_SOURCE_UUID,
    ANCS_SERVICE_UUID,
};
use ancs::control_point::EventID;

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

    // Get adapter for bluetooth
    let adapter = bluer::Session::new()
        .await
        .unwrap()
        .default_adapter()
        .await
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
    let control_point_char = find_characteristic(&iphone, ANCS_SERVICE_UUID, CONTROL_POINT_UUID)
        .await
        .unwrap();
    let data_source_char = find_characteristic(&iphone, ANCS_SERVICE_UUID, DATA_SOURCE_UUID)
        .await
        .unwrap();

    let (notification_event_tx, mut notification_event_rx) = mpsc::channel(64);
    let (notification_attributes_tx, mut notification_attributes_rx) = mpsc::channel(64);
    let (app_attributes_tx, mut app_attributes_rx) = mpsc::channel(64);

    let ns_listner = tokio::spawn(ancs::notification_source::listener(
        notification_source_char,
        notification_event_tx,
    ));

    let ds_listener = tokio::spawn(ancs::data_source::listener(
        data_source_char,
        notification_attributes_tx,
        app_attributes_tx,
    ));

    libnotify::init("ancs");

    let mut notifications: HashMap<u32, ANCSNotification> = HashMap::new();
    let mut display_names: HashMap<String, String> = HashMap::new();

    // Main thread code
    loop {
        match notification_event_rx.recv().await {
            Some(ns_event) => {
                if ns_event.event_id == EventID::NotificationAdded as u8 {
                    notifications.insert(
                        ns_event.notification_id,
                        ANCSNotification::new(),
                    );
                } else if ns_event.event_id == EventID::NotificationRemoved as u8 {
                    notifications.remove(&ns_event.notification_id).unwrap();
                }
                println!(
                    "{} {} {} {} {}",
                    ns_event.event_id,
                    ns_event.event_flags,
                    ns_event.category_id,
                    ns_event.category_count,
                    ns_event.notification_id
                );
            }
            None => continue,
        }
    }
}

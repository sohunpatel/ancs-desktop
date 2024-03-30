mod ancs;
mod utils;

use config::Config;
use log::{debug, error, info};
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
    let config_path_exists = xdg_dirs.find_config_file("ancs.toml");
    if config_path_exists.is_none() {
        error!("Cannot find config file");
        return;
    }
    let config_path = config_path_exists.unwrap();
    // Load config
    let config = Config::builder()
        .add_source(config::File::with_name(config_path.to_str().unwrap()))
        .build()
        .unwrap();

    // Initialize env_logger
    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(log::LevelFilter::Debug).init();
    // env_logger::init();
    info!("Starting ANCS application ...");

    // Get iphone device using MAC address from config
    let address_exists = config.get_string("address");
    if address_exists.is_err() {
        error!("Config does not provide MAC of iPhone");
        return;
    }
    let address = address_exists.unwrap();
    let iphone_exists = bluer::Session::new()
        .await
        .unwrap()
        .default_adapter()
        .await
        .unwrap()
        .device(bluer::Address::from_str(&address).unwrap());
    if iphone_exists.is_err() {
        error!("Cannot find your iphone ({})", address);
        return;
    }
    let iphone = iphone_exists.unwrap();
    info!(
        "Using {} for ANCS",
        config.get::<String>("address").unwrap()
    );

    // Load ANCS characteristics
    let notification_source_exists =
        find_characteristic(&iphone, ANCS_SERVICE_UUID, NOTIFICATION_SOURCE_UUID).await;
    if notification_source_exists.is_none() {
        error!("Cannot find notification source characteristic");
        return;
    }
    let notification_source_char = notification_source_exists.unwrap();
    let control_point_exists =
        find_characteristic(&iphone, ANCS_SERVICE_UUID, CONTROL_POINT_UUID).await;
    if control_point_exists.is_none() {
        error!("Cannot find control point characteristic");
        return;
    }
    let _control_point_char = control_point_exists.unwrap();
    let data_source_exists =
        find_characteristic(&iphone, ANCS_SERVICE_UUID, DATA_SOURCE_UUID).await;
    if data_source_exists.is_none() {
        error!("Cannot find data source characteristic");
        return;
    }
    let data_source_char = data_source_exists.unwrap();
    debug!("Found all ANCS characteristics");

    // Create message queues for application comms
    let (notification_event_tx, mut notification_event_rx) = mpsc::channel(64);
    let (notification_attributes_tx, mut notification_attributes_rx) = mpsc::channel(64);
    let (app_attributes_tx, mut app_attributes_rx) = mpsc::channel(64);

    // Spawn a listener that will handle the bluetooth message parsing for notification sources
    tokio::spawn(ancs::notification_source::listener(
        notification_source_char,
        notification_event_tx,
    ));

    // Spawn a listener that will handle the bluetooth message parsing for data sources
    tokio::spawn(ancs::data_source::listener(
        data_source_char,
        notification_attributes_tx,
        app_attributes_tx,
    ));

    libnotify::init("ancs").unwrap();

    let mut notifications: HashMap<u32, ANCSNotification> = HashMap::new();
    let mut display_names: HashMap<String, String> = HashMap::new();

    // Main thread code
    debug!("Starting main loop ...");
    loop {
        if let Some(event) = notification_event_rx.recv().await {
            if event.event_id == EventID::NotificationAdded as u8 {
                notifications.insert(event.notification_id, ANCSNotification::new());
            } else if event.event_id == EventID::NotificationRemoved as u8 {
                notifications.remove(&event.notification_id).unwrap();
            }
            info!("{}", event);
        }
        if let Some(attributes) = notification_attributes_rx.recv().await {
            info!("{}", attributes);
            let notification = notifications.get_mut(&attributes.notification_id).unwrap();
            notification.update(attributes.title, attributes.message);
        }
        if let Some(attributes) = app_attributes_rx.recv().await {
            info!("{}", attributes);
            if let Some(display_name) = attributes.display_name {
                display_names
                    .insert(attributes.app_identifier, display_name)
                    .unwrap();
            }
        }
    }
}

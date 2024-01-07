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

use ancs::{notification_source::NOTIFICATION_SOURCE_UUID, ANCS_SERVICE_UUID};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Load config
    let config = Config::builder()
        .add_source(config::File::with_name("ancs"))
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

    let (notification_event_tx, mut notification_event_rx) = mpsc::channel(64);

    let ns_listner = tokio::spawn(ancs::notification_source::listener(
        notification_source_char,
        notification_event_tx,
    ));

    // Main thread code
    loop {
        match notification_event_rx.recv().await {
            Some(ns_event) => {
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

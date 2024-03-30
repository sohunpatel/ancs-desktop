use bluer::{gatt::remote::Characteristic, Device, Uuid};
use log::debug;

pub async fn find_characteristic(
    device: &Device,
    service_uuid: Uuid,
    char_uuid: Uuid,
) -> Option<Characteristic> {
    debug!("Services: {:#?}", device.services().await.unwrap());
    for service in device.services().await.unwrap() {
        debug!("Found service: {}", service.uuid().await.unwrap());
        if service.uuid().await.unwrap() == service_uuid {
            for char in service.characteristics().await.unwrap() {
                debug!(
                    "{} has characteristic {}",
                    service_uuid,
                    char.uuid().await.unwrap()
                );
                if char.uuid().await.unwrap() == char_uuid {
                    return Some(char);
                }
            }
        }
    }
    None
}

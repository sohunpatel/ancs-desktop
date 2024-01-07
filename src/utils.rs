use bluer::{gatt::remote::Characteristic, Device, Uuid};

pub async fn find_characteristic(
    device: &Device,
    service_uuid: Uuid,
    char_uuid: Uuid,
) -> Option<Characteristic> {
    for service in device.services().await.unwrap() {
        if service.uuid().await.unwrap() == service_uuid {
            for char in service.characteristics().await.unwrap() {
                if char.uuid().await.unwrap() == char_uuid {
                    return Some(char);
                }
            }
        }
    }
    None
}

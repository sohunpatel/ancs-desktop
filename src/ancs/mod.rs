pub mod control_point;
pub mod data_source;
pub mod notification;
pub mod notification_source;

use bluer::Uuid;

pub const ANCS_SERVICE_UUID: Uuid = Uuid::from_u128(0x7905F431B5CE4E99A40F4B1E122D00D0);

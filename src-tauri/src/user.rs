/// Empty placeholders where users can add their logic
use crate::xap::hid::{XAPClient, XAPDevice};

use uuid::Uuid;
use xap_specs::protocol::BroadcastRaw;

// Storage for user data
#[derive(Default)]
pub struct UserData {}

pub(crate) fn pre_init() {}

pub(crate) fn on_close(_client: &XAPClient, _user_data: &mut UserData) {}

pub(crate) fn new_device(_device: &XAPDevice, _user_data: &mut UserData) {}

pub(crate) fn removed_device(_id: &Uuid, _user_data: &mut UserData) {}

pub(crate) fn broadcast_callback(
    _broadcast: BroadcastRaw,
    _device: &XAPDevice,
    _user_data: &mut UserData,
) {
}

pub(crate) fn housekeeping(_client: &XAPClient, _user_data: &mut UserData) {}

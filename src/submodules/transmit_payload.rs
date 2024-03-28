use libmtp_rs::{device::{raw::detect_raw_devices, StorageSort}, storage::Parent};

use crate::Flasher;

impl Flasher {
    pub fn transmit_payload(&mut self) {

        for device in detect_raw_devices().unwrap() {
            for allowed_devices in &self.data.devices {
                if u64::from(device.device_entry().product_id) == allowed_devices.2 {
                    let mut opened_device = device.open_uncached().unwrap();
                    opened_device.update_storage(StorageSort::NotSorted).unwrap();
                    let storage = opened_device.storage_pool();
                    println!("{:?}", storage.files_and_folders(Parent::Root));
                }
            }
        }
    }
    
}

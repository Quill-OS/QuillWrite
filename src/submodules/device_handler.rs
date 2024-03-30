use std::{io::Write, path::Path, process::Command};

use libmtp_rs::{
    device::{raw::detect_raw_devices, MtpDevice, StorageSort},
    internals::DeviceEntry,
    storage::{files::FileMetadata, Parent},
    util::CallbackReturn,
};

use crate::Flasher;

impl Flasher {
    fn detect_devices(&mut self) {
        match rusb::devices() {
            Ok(devices) => {
                for device in devices.iter() {
                    let device_desc = device.device_descriptor().unwrap();
                    for allowed_devices in &self.data.devices {
                        if u64::from(device_desc.product_id()) == allowed_devices.2 {
                            self.data.device = (
                                allowed_devices.0.clone(),
                                allowed_devices.1.clone(),
                                allowed_devices.2,
                            )
                        }
                    }
                }
            }
            Err(..) => self
                .data
                .logs
                .push_str("QuillWrite: No usb devices detected.\n"),
        }
    }

    pub fn transmit_payload(&mut self) {
        self.detect_devices();
        match detect_raw_devices() {
            Ok(raw_devices) => {
                for device in raw_devices {
                    let device_desc = device.device_entry();
                    if u64::from(device_desc.product_id) == self.data.device.2 {
                        match device.open_uncached() {
                            Some(opened_device) => {
                                Flasher::push_data(self, opened_device, device_desc);
                            }
                            None => {
                                self.data.logs.push_str(format!("QuillWrite: Device {}:{} could not be opened, it is likely in use, attempting to gain control.\n", device_desc.vendor_id, device_desc.product_id).as_str());

                                //kill common steallers of mtp devices
                                #[cfg(target_os = "linux")]
                                Command::new("pkill").arg("kiod6").output().ok();
                                #[cfg(target_family = "unix")]
                                Command::new("pkill").arg("android-file").output().ok();
                                #[cfg(target_os = "linux")]
                                Command::new("gio")
                                    .arg("mount")
                                    .arg("-u")
                                    .arg(format!(
                                        "mtp://[usb:{},{}]/",
                                        device.bus_number(),
                                        device.dev_number()
                                    ))
                                    .output()
                                    .ok();
                                match device.open_uncached() {
                                    Some(opened_device) => {
                                        Flasher::push_data(self, opened_device, device_desc);
                                    }
                                    None => {
                                        self.data.logs.push_str(
                                            format!(
                                                "QuillWrite: Could not obtain access to {}:{}.\n",
                                                device_desc.vendor_id, device_desc.product_id
                                            )
                                            .as_str(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(..) => self
                .data
                .logs
                .push_str("QuillWrite: No mtp devices detected.\n"),
        }
    }
    fn push_data(&mut self, mut opened_device: MtpDevice, device_desc: DeviceEntry) {
        println!("push quilload to device");
        opened_device
            .update_storage(StorageSort::NotSorted)
            .unwrap();
        let storage_pool = opened_device.storage_pool();
        self.data.logs.push_str(
            format!(
                "QuillWrite: Device {}:{} opened.\n",
                device_desc.vendor_id, device_desc.product_id
            )
            .as_str(),
        );
        if self.data.quilloadavailable {
            let path = Path::new(&dirs::cache_dir().unwrap())
                .join("QuillWrite")
                .join("quilload");

            let file = std::fs::File::open(path.clone()).unwrap();
            let metadata = file.metadata().unwrap();
            let mtp_metadata = FileMetadata {
                file_name: path.file_name().unwrap().to_str().unwrap(),
                file_size: metadata.len(),
                file_type: libmtp_rs::object::filetypes::Filetype::Text,
                modification_date: metadata.modified().unwrap().into(),
            };

            if let Some((_, storage)) = storage_pool.iter().next() {
                let files = storage.files_and_folders(Parent::Root);
                let mut quilload_already_present = false;
                for file in files {
                    if file.name() == "quilload" {
                        quilload_already_present = true
                    }
                }
                if !quilload_already_present {
                    let path = Path::new(&dirs::cache_dir().unwrap())
                        .join("QuillWrite")
                        .join("quilload");
                    if storage
                        .send_file_from_path_with_callback(
                            path,
                            Parent::Root,
                            mtp_metadata,
                            |sent, total| {
                                print!("\rProgress {}/{}", sent, total);
                                std::io::stdout().lock().flush().expect("Failed to flush");
                                CallbackReturn::Continue
                            },
                        )
                        .is_err()
                    {
                        self.data
                            .logs
                            .push_str("QuillWrite: Could not write to device.\n")
                    } else {
                        self.data
                            .logs
                            .push_str("QuillWrite: Quilload has been successfully sent.\n")
                    };
                } else {
                    self.data
                        .logs
                        .push_str("QuillWrite: Quilload is already on the device.\n")
                }
            } else {
                self.data
                    .logs
                    .push_str("QuillWrite: Could not access storage.\n")
            };
        } else {
            self.data
                .logs
                .push_str("QuillWrite: Could not find QuilLoad.\n")
        }
    }
}

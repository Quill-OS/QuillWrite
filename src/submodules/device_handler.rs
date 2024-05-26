use std::fs::{self, File, OpenOptions};
use std::io::Write;

use block_utils::{get_block_dev_property, get_block_devices, get_mountpoint};

use crate::Flasher;

impl Flasher {
    fn detect_devices(&mut self) {
        if let Ok(devices) = get_block_devices() {
            for device_path in devices {
                let dev_info = get_block_dev_property(device_path.clone(), "ID_MODEL_ID");
                match dev_info {
                    Ok(id) => {
                        if id.is_some() {
                            let id = id.unwrap();
                            for allowed_device in &self.data.devices {
                                if allowed_device.2.to_string() == id {
                                    if let Ok(dev) = get_mountpoint(device_path.clone()) {
                                        if let Some(mountpoint) = dev {
                                            println!("{:?}", mountpoint);
                                            self.data.device = (
                                                allowed_device.0.clone(),
                                                allowed_device.1.clone(),
                                                allowed_device.2.clone(),
                                            );
                                            self.data.mountpoint = mountpoint;
                                        } else {
                                            self.data.logs.push_str(
                                                "QuillWrite: Block device is not mounted.\n",
                                            )
                                        }
                                    } else {
                                        self.data
                                            .logs
                                            .push_str("QuillWrite: Block device has disappeared.\n")
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => self
                        .data
                        .logs
                        .push_str("QuillWrite: Could not access block device.\n"),
                }
            }
        }
    }

    pub fn transmit_payload(&mut self) {
        self.detect_devices();

        println!("{:?}", self.data.device);
        println!("{:?}", self.data.mountpoint);
        // move quilload to device
    }
    pub fn install_nickelmenu(&mut self) {
        self.detect_devices();
        let path = self.data.mountpoint.join(".kobo");
        println!("{:?}", path);
        if fs::copy(
            self.data.cache_path.join("KoboRoot.tgz"),
            path.join("KoboRoot.tgz"),
        )
        .is_err()
        {
            self.data
                .logs
                .push_str("QuillWrite: Could not load nickelmenu onto device.\n")
        }
        let config_folder_path = self.data.mountpoint.join(".adds").join("nm");
        if fs::create_dir_all(&config_folder_path).is_err() {
            self.data
                .logs
                .push_str("QuillWrite: Could not make nickelmenu path.\n")
        }
        let config_path = config_folder_path.join("config");
        if fs::read_to_string(&config_path).is_ok_and(|string| string.contains("quilload")) {
            self.data
                .logs
                .push_str("QuillWrite: Nickelmenu entry for quilload already exists.\n")
        } else {
            let mut config_file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(config_path)
                .unwrap();
            if write!(config_file, "menu_item :main    :QuilLoad           :cmd_spawn          :/usr/bin/quilload >> /mnt/onboard/.adds/quilload.log\n").is_err() {
                    self.data
                .logs
                .push_str("QuillWrite: Could not add to config file.\n")
        }
        }
    }
}

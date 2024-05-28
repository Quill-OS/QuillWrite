use std::fs::{self, OpenOptions};
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
                                if allowed_device.2 == id {
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

    pub fn transmit_payload(&mut self) -> Result<(), &'static str> {
        self.detect_devices();
        let path = self.data.mountpoint.join(".kobo");
        println!("{:?}", path);
        if fs::copy(
            self.data.cache_path.join("KoboRoot.tgz"),
            path.join("KoboRoot.tgz"),
        )
        .is_err()
        {
            return Err("QuillWrite: Could not push payload onto device.\n");
        }
        let config_folder_path = self.data.mountpoint.join(".adds").join("nm");
        if !config_folder_path.exists() && fs::create_dir_all(&config_folder_path).is_err() {
            return Err("QuillWrite: Could not make nickelmenu config path.\n");
        }
        let config_path = config_folder_path.join("config");
        if fs::read_to_string(&config_path).is_ok_and(|string| string.contains("quilload")) {
            self.data
                .logs
                .push_str("QuillWrite: Nickelmenu entry for quilload already exists.\n")
        } else {
            let mut config_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(config_path)
                .unwrap();
            if writeln!(config_file, "menu_item :main    :QuilLoad           :cmd_spawn          :/usr/bin/quilload >> /mnt/onboard/.adds/quilload.log").is_err() {
                return Err("QuillWrite: Could not add entry to nickelmenu config file.\n")
            }
        }
        if self
            .data
            .mountpoint
            .join(".adds")
            .join("quillconfig")
            .exists()
            || fs::create_dir(self.data.mountpoint.join(".adds").join("quillconfig")).is_ok()
        {
            let mut ip_addr_file_handler = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(
                    self.data
                        .mountpoint
                        .join(".adds")
                        .join("quillconfig")
                        .join("ip_address.conf"),
                )
                .unwrap();
            if let Ok(local_ip) = local_ip_address::local_ip() {
                if write!(ip_addr_file_handler, "{}", local_ip).is_err() {
                    self.data
                        .logs
                        .push_str("QuillWrite: Could not write local ip address.\n")
                }
            } else {
                self.data
                    .logs
                    .push_str("QuillWrite: Not connected to a network.\n")
            }
        } else {
            return Err("QuillWrite: Could not make the quillconfig folder.\n");
        }
        Ok(())
    }
}

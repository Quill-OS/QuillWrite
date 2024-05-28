use std::{
    fs::{self, File},
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread,
};

use crate::egui::{Color32, Rounding, Stroke};
use curl::easy::Easy;
use eframe::{
    egui::{self, TextBuffer},
    CreationContext,
};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use tar::Archive;
mod submodules;

#[derive(Serialize, Deserialize)]
struct FlasherPrefs {
    dark_mode: bool,
}

impl Default for FlasherPrefs {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}

#[derive(Default)]
struct FlasherData {
    device: (String, String, String),
    mountpoint: PathBuf,
    cache_path: PathBuf,
    logs: String,
    devices: Vec<(String, String, String)>,
    quilloadavailable: bool,
    quilloaded: bool,
    rx: Option<Receiver<bool>>,
}

#[derive(Default)]
struct Flasher {
    prefs: FlasherPrefs,
    data: FlasherData,
}

impl Flasher {
    fn new(cc: &CreationContext) -> Flasher {
        // apply egui styling
        apply_styling(cc);

        // load user preferences
        let saved_prefs: FlasherPrefs =
            confy::load("QuillWrite", Some("user_preferences")).unwrap_or_default();

        if !saved_prefs.dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        let mut data = FlasherData::default();

        // load device list
        let device_list = fs::read_to_string("./devices.json").unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&device_list).expect("JSON does not have correct format.");

        for (device, info) in json.as_object().unwrap() {
            data.devices.push((
                device.to_string(),
                info["deviceName"].to_string(),
                info["productId"].to_string(),
            ));
        }

        let cache_dir = Path::new(&dirs::cache_dir().unwrap()).join("QuillWrite");
        let quilload_path = cache_dir.join("quilload");
        if cache_dir.exists() {
            data.quilloadavailable = quilload_path.exists();
        } else {
            let _ = fs::create_dir_all(&cache_dir);
            data.quilloadavailable = false;
        }
        data.cache_path = cache_dir;
        data.quilloaded = false;
        Flasher::prepare_payload(&mut data);

        let (tx, rx) = mpsc::channel();
        // Server for recieving backup
        thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind("0.0.0.0:33") {
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            println!("New connection: {}", stream.peer_addr().unwrap());
                            thread::spawn(move || {
                                // connection succeeded
                                let mut file = File::create("/home/spagett/backup.dd.gz").unwrap();
                                std::io::copy(&mut stream, &mut file).unwrap();
                                println!("File transfer complete");
                                // handle_client(stream)
                            });
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            /* connection failed */
                        }
                    }
                }
            } else {
                eprintln!("Device is not connected to the internet");
            }
        });
        data.rx = Some(rx);

        Flasher::configure_fonts(cc);
        Flasher {
            prefs: saved_prefs,
            data,
        }
    }
    fn download_dependancies(
        cache_path: &mut PathBuf,
        dep: &'static str,
    ) -> Result<(), &'static str> {
        let (depurl, file) = if dep == "NickelMenu" {
            (
                "https://github.com/pgaskin/NickelMenu/releases/download/v0.5.4/KoboRoot.tgz",
                File::create(cache_path.join("NickelMenu.tgz")),
            )
        } else {
            (
                "https://github.com/shermp/NickelDBus/actions/runs/8863931069/artifacts/1453892074",
                File::create(cache_path.join("NickelDBus.tgz")),
            )
        };
        // if let Ok(mut tarfile) = file {
        let mut tarfile = file.unwrap();
        let mut transfer_handler = Easy::new();
        transfer_handler.follow_location(true).unwrap();
        transfer_handler.url(depurl).unwrap();
        transfer_handler
            .write_function(move |data| {
                tarfile.write_all(data).unwrap();
                Ok(data.len())
            })
            .unwrap();
        transfer_handler.perform().unwrap();
        println!("{}", transfer_handler.response_code().unwrap());
        // }
        //         // if transfer_handler.url(depurl).is_ok() {
        //         transfer_handler.write_function(|data| {
        //                 tarfile.write_all(data).unwrap();
        //             }
        //             Ok(data.len())
        //
        //         });
        // }
        // let mut Ok(file) =
        // transfer_handler.write_function(|data| {
        //
        // })
        // } else {
        // return Err("Could not contact url.");
        // }
        Ok(())
    }
    fn prepare_payload(data: &mut FlasherData) {
        if let Err(err) = Flasher::make_payload(&mut data.cache_path.clone()) {
            if err.as_str().to_lowercase().contains("Could") {
                data.logs
                    .push_str(format!("{:?}: This is likely a permissions issue", err).as_str());
            } else if err.as_str().to_lowercase().contains("found") {
                data.logs.push_str(
                    format!(
                        "{:?}: It is likely not downloaded, attempting to fetch.",
                        err
                    )
                    .as_str(),
                );
                if err.as_str().contains("NickelMenu") {
                    if let Err(err) =
                        Flasher::download_dependancies(&mut data.cache_path, "NickelMenu")
                    {
                        data.logs
                            .push_str(format!("Could not download NickelMenu: {:?}", err).as_str())
                    }
                    Flasher::prepare_payload(data);
                } else if err.as_str().contains("NickelDBus") {
                    data.logs
                        .push_str("Can not download NickelMenu as it is a github artifact.");
                    // if let Err(err) =
                    // Flasher::download_dependancies(&mut data.cache_path, "NickelDBus")
                    // {
                    //     data.logs
                    //         .push_str(format!("Could not download NickelMenu: {:?}", err).as_str())
                    // }
                    // Flasher::prepare_payload(data);
                }
            }
        }
    }
    fn make_payload(cache_path: &mut PathBuf) -> Result<(), &'static str> {
        // Remove existing old koboroot files
        let koboroot_dir = cache_path.join("KoboRoot");
        let quilload = cache_path.join("quilload");
        if fs::remove_dir_all(&koboroot_dir).is_ok() || !koboroot_dir.exists() {
            if fs::remove_file(cache_path.join("KoboRoot.tgz")).is_ok()
                || !cache_path.join("KoboRoot.tgz").exists()
            {
                // Open tar archives
                if let Ok(nickle_menu) = File::open(cache_path.join("NickelMenu.tgz")) {
                    if let Ok(nickle_dbus) = File::open(cache_path.join("NickelDBus.tgz")) {
                        // Make future koboroot folder
                        if koboroot_dir.exists() || fs::create_dir(&koboroot_dir).is_ok() {
                            // Decompress archives
                            let nickle_menu_tar = GzDecoder::new(nickle_menu);
                            let nickle_dbus_tar = GzDecoder::new(nickle_dbus);
                            // Define archives
                            let mut nickel_menu_archive = Archive::new(nickle_menu_tar);
                            let mut nickel_dbus_archive = Archive::new(nickle_dbus_tar);
                            // Extract archives
                            if nickel_menu_archive.unpack(&koboroot_dir).is_ok() {
                                if nickel_dbus_archive.unpack(&koboroot_dir).is_ok() {
                                    // Move quilload into future archive
                                    if fs::copy(
                                        quilload,
                                        cache_path
                                            .join("KoboRoot")
                                            .join("usr")
                                            .join("bin")
                                            .join("quilload"),
                                    )
                                    .is_ok()
                                    {
                                        // Create new tarball
                                        if let Ok(koboroot) =
                                            File::create(cache_path.join("KoboRoot.tgz"))
                                        {
                                            let encryption =
                                                GzEncoder::new(koboroot, Compression::fast());
                                            let mut tar = tar::Builder::new(encryption);
                                            if tar
                                                .append_dir_all("", cache_path.join("KoboRoot"))
                                                .is_err()
                                            {
                                                return Err("Could not put files into tar archive");
                                            }
                                        } else {
                                            return Err("Could not create KoboRoot.tgz");
                                        }
                                    } else {
                                        return Err("Could not place quilload into KoboRoot");
                                    }
                                } else {
                                    return Err("Could not extract NDBus archive to KoboRoot");
                                }
                            } else {
                                return Err("Could not extract NM archive to KoboRoot");
                            }
                        } else {
                            return Err("KoboRoot does not exist and could not be made");
                        }
                    } else {
                        return Err("NickelDBus.tgz not found");
                    }
                } else {
                    return Err("NickelMenu.tgz not found");
                };
            } else {
                return Err("Could not remove KoboRoot.tgz file");
            }
        } else {
            return Err("Could not remove KoboRoot directory");
        }
        Ok(())
    }
}

impl eframe::App for Flasher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // always repaint to have accurate device detection
        ctx.request_repaint();
        ctx.set_pixels_per_point(1.8);
        Flasher::render_header(self, ctx);

        if self.data.quilloaded {
            Flasher::panel_post_send(self, ctx);
        } else {
            Flasher::panel_pre_send(self, ctx);
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            title: Some("QuillWrite".to_string()),
            decorations: Some(true),
            ..Default::default()
        },

        ..Default::default()
    };
    eframe::run_native(
        "QuillWrite",
        options,
        Box::new(|cc| Box::new(Flasher::new(cc))),
    )
    .expect("Could not launch eframe, you may have a driver error.");
}

fn apply_styling(cc: &CreationContext) {
    let mut style: egui::Style = (*cc.egui_ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(5.0, 10.0);

    cc.egui_ctx.set_style(style);
    let new_style = egui::style::WidgetVisuals {
        bg_fill: Color32::from_rgb(17, 17, 17),
        weak_bg_fill: Color32::from_rgb(17, 17, 17),

        rounding: Rounding {
            nw: 4.,
            ne: 4.,
            sw: 4.,
            se: 4.,
        },

        bg_stroke: Stroke {
            width: 1.,
            color: Color32::from_rgb(140, 140, 140),
        },
        fg_stroke: Stroke {
            width: 1.,
            color: Color32::from_rgb(140, 140, 140),
        },

        expansion: 2.,
    };
    let new_hovered_style = egui::style::WidgetVisuals {
        bg_fill: Color32::from_rgb(17, 17, 17),
        weak_bg_fill: Color32::from_rgb(17, 17, 17),

        rounding: Rounding {
            nw: 4.,
            ne: 4.,
            sw: 4.,
            se: 4.,
        },

        bg_stroke: Stroke {
            width: 1.5,
            color: egui::Color32::from_rgb(56, 55, 55),
        },
        fg_stroke: Stroke {
            width: 1.,
            color: Color32::from_rgb(140, 140, 140),
        },

        expansion: 2.,
    };
    cc.egui_ctx.set_visuals(egui::style::Visuals {
        widgets: egui::style::Widgets {
            active: new_style,
            inactive: new_style,
            hovered: new_hovered_style,
            noninteractive: new_style,
            open: new_hovered_style,
        },
        ..Default::default()
    });
}

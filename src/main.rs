use std::{
    fs::{self, File},
    net::TcpListener,
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
    thread::{self, sleep},
    time::Duration,
};

use crate::egui::{Color32, Rounding, Stroke};
use eframe::{egui, CreationContext};
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
    tx: Option<Sender<bool>>,
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

        if Flasher::prepare_payload(&mut data.cache_path.clone()).is_err() {
            eprintln!(
                "please make sure you have NickelDBus, quilload and NickelMenu in the cache dir"
            )
        }

        let (tx, rx) = mpsc::channel();
        // Server for recieving backup
        thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind("0.0.0.0:3333") {
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
                            println!("Error: {}", e);
                            /* connection failed */
                        }
                    }
                }
            }
        });
        data.tx = Some(tx);

        Flasher::configure_fonts(cc);
        Flasher {
            prefs: saved_prefs,
            data,
        }
    }
    fn prepare_payload(cache_path: &mut PathBuf) -> Result<(), std::io::Error> {
        // Remove existing old koboroot files
        fs::remove_dir_all(cache_path.join("KoboRoot"));
        fs::remove_dir_all(cache_path.join("KoboRoot.tgz"));
        // Open tar archives
        let nickle_menu = File::open(cache_path.join("NickelMenu.tgz"))?;
        let nickle_dbus = File::open(cache_path.join("NickelDBus.tgz"))?;

        let quilload = cache_path.join("quilload");
        // Make future koboroot folder
        fs::create_dir(cache_path.join("KoboRoot"))?;
        // Decompress archives
        let nickle_menu_tar = GzDecoder::new(nickle_menu);
        let nickle_dbus_tar = GzDecoder::new(nickle_dbus);
        // Define archives
        let mut nickel_menu_archive = Archive::new(nickle_menu_tar);
        let mut nickel_dbus_archive = Archive::new(nickle_dbus_tar);
        // Extract archives
        nickel_menu_archive.unpack(cache_path.join("KoboRoot"))?;
        nickel_dbus_archive.unpack(cache_path.join("KoboRoot"))?;
        // Move quilload into future archive
        fs::copy(
            quilload,
            cache_path
                .join("KoboRoot")
                .join("usr")
                .join("bin")
                .join("quilload"),
        )?;
        // Create new tarball
        let koboroot = File::create(cache_path.join("KoboRoot.tgz"))?;
        let encryption = GzEncoder::new(koboroot, Compression::fast());
        let mut tar = tar::Builder::new(encryption);
        tar.append_dir_all("", cache_path.join("KoboRoot"))?;
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

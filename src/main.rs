use std::fs;

use crate::egui::{Color32, Rounding, Stroke};
use eframe::{
    egui::{self},
    CreationContext,
};
use serde::{Deserialize, Serialize};
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

struct FlasherData {
    device: (String, String),
    logs: String,
    devices: Vec<(String, String)>,
}
impl Default for FlasherData {
    fn default() -> Self {
        Self {
            device: ("".to_string(), "".to_string()),
            logs: "".to_string(),
            devices: vec![],
        }
    }
}

#[derive(Default)]
struct Flasher {
    prefs: FlasherPrefs,
    data: FlasherData,
}

impl Flasher {
    fn new(cc: &CreationContext) -> Flasher {
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

        // load user preferences
        let saved_prefs: FlasherPrefs =
            confy::load("QuillWrite", Some("user_preferences")).unwrap_or_default();

        let mut data = FlasherData::default();

        // load device list
        let device_list = fs::read_to_string("./devices.json").unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&device_list).expect("JSON does not have correct format.");

        for (key, value) in json.as_object().unwrap() {
            data.devices.push((key.to_string(), value.to_string()));
        }
        if !saved_prefs.dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        Flasher::configure_fonts(cc);
        Flasher {
            prefs: saved_prefs,
            data: data,
        }
    }
}

impl eframe::App for Flasher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // always repaint to have accurate device detection
        ctx.request_repaint();
        ctx.set_pixels_per_point(1.8);

        Flasher::render_header(self, &ctx);
        Flasher::render_main_panel(self, &ctx);
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

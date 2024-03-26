use crate::{Flasher, FlasherPrefs};
use eframe::egui::{self, menu, Button, RichText, TopBottomPanel};

impl Flasher {
    pub fn render_header(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("Title Panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.vertical_centered(|ui| ui.heading("QuillWrite"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.add_space(5.);
                    if ui
                        .add(Button::new(
                            RichText::new(if self.prefs.dark_mode {
                                "  "
                            } else {
                                "  "
                            })
                            .size(25.0), // .line_height(Some(20.)),
                        ))
                        .clicked()
                    {
                        if self.prefs.dark_mode {
                            ctx.set_visuals(egui::Visuals::light());
                        } else {
                            ctx.set_visuals(egui::Visuals::dark());
                        }
                        self.prefs.dark_mode = !self.prefs.dark_mode;
                        if let Err(..) = confy::store(
                            "QuillWrite",
                            Some("userpreferences"),
                            FlasherPrefs {
                                dark_mode: self.prefs.dark_mode,
                            },
                        ) {
                            self.data
                                .logs
                                .push_str("QuillWrite : Failed to save the app state.")
                        }
                    }
                });
            })
        });
    }
}

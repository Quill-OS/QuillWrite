use eframe::egui::{self, Button, CentralPanel, CollapsingHeader, Rounding, ScrollArea};

use crate::Flasher;

impl Flasher {
    pub fn panel_pre_send(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // ui.visuals_mut().widgets.active.bg_stroke = eframe::epaint::Stroke {
                //     width: 0.,
                //     color: egui::Color32::RED,
                // };
                // ui.visuals_mut().widgets.inactive.bg_stroke = eframe::epaint::Stroke {
                //     width: 0.,
                //     color: egui::Color32::RED,
                // };
                //
                // ui.vertical(|ui| {
                //     ui.label("Select your E-Reader");
                //     egui::ComboBox::from_label(" ")
                //         .selected_text(&self.data.device.1)
                //         .show_ui(ui, |ui| {
                //             for (model_number, device_name, product_id) in self.data.devices.clone()
                //             {
                //                 ui.selectable_value(
                //                     &mut self.data.device,
                //                     (model_number, device_name.clone(), product_id),
                //                     device_name,
                //                 );
                //             }
                //         });
                // });

                ui.vertical(|ui| {
                    ui.vertical_centered(|ui| {
                        if ui.button("Install QuilLoad to ereader.").clicked() {
                            if let Err(err) = Flasher::transmit_payload(self) {
                                self.data.quilloaded = false;
                                self.data.logs.push_str(err)
                            } else {
                                self.data.quilloaded = true;
                            }
                        }
                    });
                    egui::CollapsingHeader::new("Logs")
                        .default_open(true)
                        .show_unindented(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(10.);
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(17, 17, 17))
                                    .rounding(Rounding {
                                        nw: 4.,
                                        ne: 4.,
                                        sw: 4.,
                                        se: 4.,
                                    })
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.add_space(10.);

                                            ui.horizontal(|ui| {
                                                ui.add_space(10.);
                                                if ui
                                                    .add(
                                                        Button::new("Copy Log").fill(
                                                            egui::Color32::from_rgb(27, 27, 27),
                                                        ),
                                                    )
                                                    .clicked()
                                                {
                                                    // ui.output().copied_text = self.config.logs.clone();
                                                    ui.output_mut(|i| {
                                                        i.copied_text = self.data.logs.clone()
                                                    });
                                                }
                                            });
                                            ScrollArea::vertical()
                                                .auto_shrink([false, false])
                                                .stick_to_bottom(true)
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.add_space(10.);
                                                        ui.vertical(|ui| {
                                                            ui.monospace(self.data.logs.clone());
                                                        });
                                                        ui.add_space(5.);
                                                    });
                                                });
                                        });
                                    });
                            })
                        });
                })
            });
        });
    }
    pub fn panel_post_send(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| ui.label("Watching for device."));
    }
}

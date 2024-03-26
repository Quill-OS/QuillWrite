use eframe::{
    egui::{FontData, FontDefinitions},
    epaint::FontFamily,
    CreationContext,
};

use crate::Flasher;

impl Flasher {
    pub fn configure_fonts(cc: &CreationContext) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "AgaveNerd".to_owned(),
            FontData::from_static(include_bytes!(
                "../../assets/agave/AgaveNerdFontMono-Regular.ttf"
            )),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .push("AgaveNerd".to_owned());
        cc.egui_ctx.set_fonts(fonts)
    }
}

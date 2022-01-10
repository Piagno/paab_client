use eframe::{
    egui::{CentralPanel, FontDefinitions, FontFamily, ScrollArea},
    epi::App,
    run_native, NativeOptions,
};
use serde::Deserialize;
use ureq;

struct Paab {
    trains: Vec<Train>,
}

impl App for Paab {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &eframe::epi::Frame,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_fonts(ctx)
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &eframe::epi::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for train in fetch_trains() {
                    ui.label(&train.train_number);
                    ui.label(&train.train_type);
                    ui.label(&train.departure_time);
                }
            })
        });
    }

    fn name(&self) -> &str {
        "PAAB"
    }
}

impl Paab {
    fn new() -> Paab {
        Paab {
            trains: fetch_trains(),
        }
    }
    fn configure_fonts(&self, ctx: &eframe::egui::CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Heading,
            (FontFamily::Proportional, 35.),
        );
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Body,
            (FontFamily::Proportional, 20.),
        );
        ctx.set_fonts(font_def);
    }
}

fn main() {
    let app = Paab::new();
    let win_option = NativeOptions::default();
    run_native(Box::new(app), win_option);
}

#[derive(Deserialize)]
struct Train {
    train_id: String,
    train_number: String,
    departure_time: String,
    estimated_retard: Option<String>,
    destination: String,
    drives: String,
    effective_departure_time: Option<String>,
    train_type: String,
    departure_station: String,
    normal_run_time: String,
    additional_info: Option<String>,
}

fn fetch_trains() -> Vec<Train> {
    return ureq::get("https://tool.piagno.ch/paab/api.php")
        .call()
        .unwrap()
        .into_json()
        .unwrap();
}

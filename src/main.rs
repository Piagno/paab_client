#![windows_subsystem = "windows"]
use eframe::{
    egui::{CentralPanel, FontDefinitions, FontFamily, ScrollArea},
    epi::App,
    run_native, NativeOptions,
};
use serde::Deserialize;
use std::fmt::Display;
use std::time::{Duration, SystemTime};
use ureq;

const UPDATE_RATE: Duration = Duration::from_secs(1);
const NO_RETARD: &str = "No retard";

#[derive(thiserror::Error, Debug)]
enum TrainError {
    RequestFailed(#[from] ureq::Error),
    ConvertingFailed(#[from] std::io::Error),
}

impl Display for TrainError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

struct Paab {
    updated: SystemTime,
    trains: Vec<Train>,
}

impl App for Paab {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &eframe::epi::Frame,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_fonts(ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &eframe::epi::Frame) {
        if self.updated.elapsed().expect("ERROR THINGI") >= UPDATE_RATE {
            self.updated = SystemTime::now();
            match fetch_trains() {
                Ok(trains) => self.trains = trains,
                _ => (),
            }
        }
        ctx.request_repaint();
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for train in &self.trains {
                    ui.label(&train.train_number);
                    ui.label(&train.train_type);
                    ui.label(&train.drives);
                    ui.label(&train.departure_time);
                    match &train.estimated_retard {
                        Option::Some(estimated_retard) => {
                            let estimated_retard = estimated_retard.parse().unwrap();
                            match estimated_retard {
                                0 => ui.label(NO_RETARD),
                                _ => ui.label(estimated_retard.to_string()),
                            }
                        }
                        Option::None => ui.label(NO_RETARD),
                    };
                    match &train.additional_info {
                        Option::Some(additional_info) => ui.label(additional_info),
                        Option::None => ui.label(""),
                    };
                    ui.label("Effective departure at: ");
                    match &train.effective_departure_time {
                        Option::Some(effective_departure_time) => {
                            ui.label(effective_departure_time)
                        }
                        Option::None => ui.label(""),
                    };
                    ui.separator();
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
            updated: SystemTime::now(),
            trains: fetch_trains().unwrap_or(Vec::new()),
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

fn fetch_trains() -> Result<Vec<Train>, TrainError> {
    Ok(ureq::get("https://tool.piagno.ch/paab/api.php")
        .call()?
        .into_json()?)
}

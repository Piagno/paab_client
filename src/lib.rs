#![windows_subsystem = "windows"]
use chrono::{DateTime, Utc};
use eframe::{
    egui::{
        color::Color32, CentralPanel, FontDefinitions, FontFamily, Grid, ScrollArea, Vec2, Visuals,
    },
    epi::App,
};
#[cfg(target_arch = "wasm32")]
use gloo_timers;
#[cfg(target_arch = "wasm32")]
use reqwasm;
use serde::Deserialize;
use std::{
    fmt::Display,
    sync::mpsc::{channel, Receiver},
};
#[cfg(not(target_arch = "wasm32"))]
use std::{thread, time::Duration as StdDuration};
#[cfg(not(target_arch = "wasm32"))]
use ureq;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;

const UPDATE_RATE: u32 = 15000;
const API_URL: &str = "https://tool.piagno.ch/paab/api.php";

#[derive(thiserror::Error, Debug)]
enum TrainError {
    #[cfg(not(target_arch = "wasm32"))]
    RequestFailed(#[from] ureq::Error),
    #[cfg(target_arch = "wasm32")]
    RequestFailed(#[from] reqwasm::Error),
    ConvertingFailed(#[from] std::io::Error),
    BadRequest(String),
}

impl Display for TrainError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

pub struct Paab {
    updated: DateTime<Utc>,
    trains: Vec<Train>,
    trains_rx: Option<Receiver<Vec<Train>>>,
}

impl App for Paab {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &eframe::epi::Frame,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_styles(ctx);
        let (trains_tx, trains_rx) = channel();
        self.trains_rx = Some(trains_rx);
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || loop {
            if let Ok(trains) = fetch_trains() {
                if let Err(e) = trains_tx.send(trains) {
                    panic!("Error sending news data: {}", e);
                }
            }
            thread::sleep(StdDuration::from_millis(UPDATE_RATE as u64));
        });
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(fetch_trains(trains_tx.clone()));
        #[cfg(target_arch = "wasm32")]
        gloo_timers::callback::Interval::new(UPDATE_RATE, move || {
            wasm_bindgen_futures::spawn_local(fetch_trains(trains_tx.clone()));
        })
        .forget();
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, _frame: &eframe::epi::Frame) {
        ctx.request_repaint();
        if let Some(rx) = &self.trains_rx {
            match rx.try_recv() {
                Ok(trains) => {
                    self.trains = trains;
                }
                Err(_e) => {}
            }
        }
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                Grid::new("grid")
                    .striped(true)
                    .spacing(Vec2::new(25., 10.))
                    .show(ui, |ui| {
                        for train in &self.trains {
                            ui.label(&train.train_number);
                            ui.label(&train.train_type);
                            ui.label(pretty_print_datetime(&train.departure_time));
                            match &train.effective_departure_time {
                                Option::Some(effective_departure_time) => {
                                    ui.colored_label(
                                        Color32::GREEN,
                                        format!(
                                            "{}",
                                            pretty_print_datetime(effective_departure_time)
                                        ),
                                    );
                                }
                                Option::None => match &train.drives[..] {
                                    "1" => match &train.estimated_retard {
                                        Option::Some(estimated_retard) => {
                                            let estimated_retard =
                                                estimated_retard.parse().unwrap();
                                            match estimated_retard {
                                                0 => (),
                                                _ => {
                                                    ui.colored_label(
                                                        Color32::from_rgb(255, 136, 0),
                                                        format!("{} min", estimated_retard),
                                                    );
                                                }
                                            }
                                        }
                                        Option::None => (),
                                    },
                                    "outage" => {
                                        ui.colored_label(Color32::RED, "Ausfall!");
                                    }
                                    "driven" => match &train.estimated_retard {
                                        Option::Some(estimated_retard)
                                            if estimated_retard == "0" =>
                                        {
                                            ui.colored_label(Color32::GREEN, "Gefahren");
                                        }
                                        Option::None => {
                                            ui.colored_label(Color32::GREEN, "Gefahren");
                                        }
                                        Option::Some(estimated_retard) => {
                                            ui.colored_label(
                                                Color32::GREEN,
                                                format!("{} min Verspätung", estimated_retard),
                                            );
                                        }
                                    },
                                    _ => {
                                        ui.colored_label(Color32::YELLOW, &train.drives);
                                    }
                                },
                            };
                            match &train.additional_info {
                                Option::Some(additional_info) => match &additional_info[..] {
                                    "" => (),
                                    _ => {
                                        ui.colored_label(Color32::YELLOW, additional_info);
                                    }
                                },
                                Option::None => (),
                            };
                            ui.end_row();
                        }
                    })
            });
        });
    }

    fn name(&self) -> &str {
        "PAAB"
    }
}

impl Paab {
    pub fn new() -> Paab {
        return Paab {
            updated: Utc::now(),
            trains: Vec::new(),
            trains_rx: None,
        };
    }
    fn configure_styles(&self, ctx: &eframe::egui::CtxRef) {
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
        let mut visuals = Visuals::default();
        visuals.dark_mode = true;
        ctx.set_visuals(visuals)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn fetch_trains() -> Result<Vec<Train>, TrainError> {
    Ok(ureq::get(API_URL).call()?.into_json()?)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_trains(trains_tx: std::sync::mpsc::Sender<Vec<Train>>) {
    if let Ok(trains) = fetch_trains_web().await {
        if let Err(e) = trains_tx.send(trains) {
            panic!("Error sending train data: {}", e);
        }
    } else {
        panic!("failed fetching trains");
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_trains_web() -> Result<Vec<Train>, TrainError> {
    let req = reqwasm::http::Request::get(API_URL);
    let resp = req
        .send()
        .await
        .map_err(|_| TrainError::BadRequest("failed sending request".to_string()))?;
    let response: Vec<Train> = resp
        .json()
        .await
        .map_err(|_| TrainError::BadRequest("failed converting response to json".to_string()))?;
    Ok(response)
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

fn pretty_print_datetime(datetime: &str) -> String {
    format!("{}:{}", &datetime[11..13], &datetime[14..16],)
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_web(canvas_id: &str) {
    let app = Paab::new();
    eframe::start_web(canvas_id, Box::new(app));
}

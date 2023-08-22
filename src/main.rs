use eframe::egui;
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::{Address, BlockNumber, U256, U64},
    Web3,
};

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::{Error, Write};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Arbitrage Bot",
        eframe::NativeOptions {
            drag_and_drop_support: false,
            initial_window_size: Some(egui::vec2(800.0, 600.0)),
            ..Default::default()
        },
        Box::new(|_| Box::new(App::new())),
    )
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
enum Chain {
    Ethereum,
    Binance,
    Polygon,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Chain::Ethereum => write!(f, "Ethereum"),
            Chain::Binance => write!(f, "Binance"),
            Chain::Polygon => write!(f, "Polygon"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    chain: Chain,
}

struct App {
    selected_chain: Chain,
    text_input: String,
    account_text_dropped: bool,
}

impl App {
    fn new() -> Self {
        App {
            selected_chain: match get_chain_from_config() {
                Ok(chain) => chain,
                Err(_) => Chain::Ethereum,
            },
            text_input: String::new(),
            account_text_dropped: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 20.0;
                let previous_chain = self.selected_chain.clone();

                ui.group(|ui| {
                    ui.spacing_mut().item_spacing.y = 20.0;
                    if ui.button("Start Arbitrage").clicked() {
                        println!("Button 1 was pressed!");
                    }
                    if ui.button("Stop Arbitrage").clicked() {
                        println!("Button 2 was pressed!");
                    }
                });

                ui.group(|ui| {
                    if ui.button("Settings").clicked() {
                        self.account_text_dropped = !self.account_text_dropped;
                    }
                    if self.account_text_dropped {
                        ui.horizontal(|ui| {
                            ui.label("Wallet Private Key: ");
                            ui.text_edit_singleline(&mut self.text_input);

                            egui::ComboBox::from_label("Select a chain")
                                .selected_text(self.selected_chain.clone().to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.selected_chain,
                                        Chain::Ethereum,
                                        "Ethereum",
                                    );
                                    ui.selectable_value(
                                        &mut self.selected_chain,
                                        Chain::Binance,
                                        "Binance",
                                    );
                                    ui.selectable_value(
                                        &mut self.selected_chain,
                                        Chain::Polygon,
                                        "Polygon",
                                    );
                                });
                                let selected_chain: Chain = self.selected_chain.clone();
                                if selected_chain != previous_chain {
                                    handle_chain_selection(selected_chain);
                                }
                        });
                    }
                });
            });
        });
    }
}

fn get_chain_from_config() -> Result<Chain, Error> {
    let data = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&data)?;

    Ok(config.chain)
}

fn handle_chain_selection(selected_chain: Chain) {
    let mut config: Config;

    if Path::new("config.json").exists() {
        let data = fs::read_to_string("config.json").expect("Failed to read config");
        config = serde_json::from_str(&data).expect("Error parsing JSON data");
    } else {
        config = Config {
            chain: selected_chain,
        };
    }

    config.chain = selected_chain;
    let json_data = serde_json::to_string_pretty(&config).expect("Failed to serialize to JSON");
    let mut file = File::create("config.json").expect("Failed to open file");
    file.write_all(json_data.as_bytes())
        .expect("Failed to write data");

    println!("Option chosen: {:#?}", selected_chain);
}
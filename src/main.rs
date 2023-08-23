use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{self, File},
    io::{Error, Write},
    path::Path,
};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::{Address, BlockNumber, U256, U64},
    Web3,
    signing::SecretKey,
};

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
    private_key: String,
    token_address_1: String,
    token_address_2: String,
}

struct App {
    selected_chain: Chain,
    private_key_input: String,
    token_address_input_1: String,
    token_address_input_2: String,
    temp: TempValues,
    account_text_dropped: bool,
}

struct TempValues {
    temp_private_key_input: String,
    temp_token_address_input_1: String,
    temp_token_address_input_2: String,
    temp_selected_chain: String,
}

impl App {
    fn new() -> Self {
        App {
            selected_chain: match get_chain_from_config() {
                Ok(chain) => chain,
                Err(_) => Chain::Ethereum,
            },
            private_key_input: String::new(),
            account_text_dropped: false,
            token_address_input_1: String::new(),
            token_address_input_2: String::new(),
            temp: TempValues {
                temp_private_key_input: String::new(),
                temp_token_address_input_1: String::new(),
                temp_token_address_input_2: String::new(),
                temp_selected_chain: String::new(),
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 20.0;
                
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
                            ui.text_edit_singleline(&mut self.temp.temp_private_key_input);

                            egui::ComboBox::from_label("Select a chain")
                                .selected_text(self.temp.temp_selected_chain.clone())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.temp.temp_selected_chain, "Ethereum".to_string(), "Ethereum");
                                    ui.selectable_value(&mut self.temp.temp_selected_chain, "Polygon".to_string(), "Polygon");
                                    ui.selectable_value(&mut self.temp.temp_selected_chain, "Binance".to_string(), "Binance");
                                });
                        });

                        ui.label("Addresses of the tokens to trade");

                        ui.horizontal(|ui| {
                            ui.label("Address: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_1);
                            ui.label("Address: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_2);
                        });

                        if ui.button("Save").clicked() {
                            self.private_key_input = self.temp.temp_private_key_input.clone();
                            self.token_address_input_1 = self.temp.temp_token_address_input_1.clone();
                            self.token_address_input_2 = self.temp.temp_token_address_input_2.clone();

                            self.selected_chain = match self.temp.temp_selected_chain.as_str() {
                                "Ethereum" => Chain::Ethereum,
                                "Polygon" => Chain::Polygon,
                                "Binance" => Chain::Binance,
                                _ => Chain::Ethereum,
                            };

                            let config = Config {
                                chain: self.selected_chain.clone(),
                                private_key: self.private_key_input.clone(),
                                token_address_1: self.token_address_input_1.clone(),
                                token_address_2: self.token_address_input_2.clone(),
                            };
                            write_config(config);
                        }
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

fn write_config(config: Config) {
    let json_data = serde_json::to_string_pretty(&config).expect("Failed to serialize to JSON");
    let mut file = File::create("config.json").expect("Failed to open file");
    file.write_all(json_data.as_bytes())
        .expect("Failed to write data");
}
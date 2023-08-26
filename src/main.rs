use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    fs::{self, File},
    io::{Error, Write},
    str::FromStr,
};
use web3::{
    contract::{Contract, Options},
    signing::SecretKey,
    transports::Http,
    types::{Address, BlockNumber, H160, U256},
    Web3,
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Default)]
enum Chain {
    #[default]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Config {
    chain: Chain,
    contract_address: String,
    private_key: String,
    token_address_1: String,
    token_address_2: String,
    token_address_3: String,
    token_address_4: String,
}

impl Config {
    fn default() -> Self {
        Config {
            chain: Chain::Ethereum,
            contract_address: String::new(),
            private_key: String::new(),
            token_address_1: String::new(),
            token_address_2: String::new(),
            token_address_3: String::new(),
            token_address_4: String::new(),
        }
    }
}

struct App {
    selected_chain: Chain,
    private_key_input: String,
    token_address_input_1: String,
    token_address_input_2: String,
    token_address_input_3: String,
    token_address_input_4: String,
    contract_address: String,
    temp: TempValues,
    account_text_dropped: bool,
    invalid_address_popup: bool,
}

struct TempValues {
    temp_private_key_input: String,
    temp_token_address_input_1: String,
    temp_token_address_input_2: String,
    temp_token_address_input_3: String,
    temp_token_address_input_4: String,
    temp_selected_chain: Chain,
    temp_contract_address: String,
}

impl TempValues {
    fn default() -> Self {
        TempValues {
            temp_private_key_input: String::new(),
            temp_token_address_input_1: String::new(),
            temp_token_address_input_2: String::new(),
            temp_token_address_input_3: String::new(),
            temp_token_address_input_4: String::new(),
            temp_selected_chain: Chain::default(),
            temp_contract_address: String::new(),
        }
    }
}

impl TempValues {
    fn new(config: Config) -> Self {
        TempValues {
            temp_private_key_input: config.private_key,
            temp_contract_address: config.contract_address,
            temp_selected_chain: config.chain,
            temp_token_address_input_1: config.token_address_1,
            temp_token_address_input_2: config.token_address_2,
            temp_token_address_input_3: config.token_address_3,
            temp_token_address_input_4: config.token_address_4,
        }
    }
}

impl App {
    fn default() -> Self {
        App {
            selected_chain: Chain::Ethereum,
            private_key_input: String::new(),
            account_text_dropped: false,
            token_address_input_1: String::new(),
            token_address_input_2: String::new(),
            token_address_input_3: String::new(),
            token_address_input_4: String::new(),
            contract_address: String::new(),
            invalid_address_popup: false,
            temp: TempValues::default(),
        }
    }
}

impl App {
    fn new() -> Self {
        let current_config = get_config();

        match current_config {
            Ok(config) => {
                let config2 = config.clone();
                App {
                    selected_chain: config.chain,
                    private_key_input: config.private_key,
                    account_text_dropped: false,
                    token_address_input_1: config.token_address_1,
                    token_address_input_2: config.token_address_2,
                    token_address_input_3: config.token_address_3,
                    token_address_input_4: config.token_address_4,
                    contract_address: config.contract_address,
                    invalid_address_popup: false,
                    temp: TempValues::new(config2),
                }
            }
            Err(_) => return App::default(),
        }
    }
}

impl App {
    fn get_config() -> Config {
        let current_config = get_config();

        match current_config {
            Ok(config) => Config {
                chain: config.chain,
                contract_address: config.contract_address,
                private_key: config.private_key,
                token_address_1: config.token_address_1,
                token_address_2: config.token_address_2,
                token_address_3: config.token_address_3,
                token_address_4: config.token_address_4,
            },
            Err(_) => return Config::default(),
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
                        begin_arbitrage(self);
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
                        let combo_box_width = 200.0;
                        let indent = (ui.available_width() - combo_box_width) / 2.0;

                        ui.horizontal(|ui| {
                            ui.add_space(indent);
                            egui::ComboBox::from_label("Select a chain")
                                .selected_text(self.temp.temp_selected_chain.clone().to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.temp.temp_selected_chain,
                                        Chain::Ethereum,
                                        "Ethereum",
                                    );
                                    ui.selectable_value(
                                        &mut self.temp.temp_selected_chain,
                                        Chain::Polygon,
                                        "Polygon",
                                    );
                                    ui.selectable_value(
                                        &mut self.temp.temp_selected_chain,
                                        Chain::Binance,
                                        "Binance",
                                    );
                                });
                        });

                        ui.label("Exchange Router Contract Address :");
                        ui.text_edit_singleline(&mut self.temp.temp_contract_address);

                        ui.label("Addresses of tokens for both trading pairs");
                        ui.horizontal(|ui| {
                            ui.label("Token Address 1: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_1);
                            ui.label("Token Address 2: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_2);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Token Address 1: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_3);
                            ui.label("Token Address 2: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_4);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Wallet Private Key: ");
                            ui.text_edit_singleline(&mut self.temp.temp_private_key_input);
                        });

                        if ui.button("Save").clicked() {
                            if !self.temp.temp_private_key_input.is_empty() {
                                self.private_key_input = self.temp.temp_private_key_input.clone();
                            }
                            if !self.temp.temp_token_address_input_1.is_empty() {
                                self.token_address_input_1 =
                                    self.temp.temp_token_address_input_1.clone();
                            }
                            if !self.temp.temp_token_address_input_2.is_empty() {
                                self.token_address_input_2 =
                                    self.temp.temp_token_address_input_2.clone();
                            }
                            if !self.temp.temp_token_address_input_3.is_empty() {
                                self.token_address_input_3 =
                                    self.temp.temp_token_address_input_3.clone();
                            }
                            if !self.temp.temp_token_address_input_4.is_empty() {
                                self.token_address_input_4 =
                                    self.temp.temp_token_address_input_4.clone();
                            }
                            if !self.temp.temp_contract_address.is_empty() {
                                self.contract_address = self.temp.temp_contract_address.clone();
                            }
                            self.selected_chain = self.temp.temp_selected_chain;

                            let config = Config {
                                chain: self.selected_chain.clone(),
                                contract_address: self.contract_address.clone(),
                                private_key: self.private_key_input.clone(),
                                token_address_1: self.token_address_input_1.clone(),
                                token_address_2: self.token_address_input_2.clone(),
                                token_address_3: self.token_address_input_3.clone(),
                                token_address_4: self.token_address_input_4.clone(),
                            };
                            write_config(config);
                        }
                    }
                });
                if self.invalid_address_popup {
                    egui::Window::new("Invalid Address").show(ctx, |ui| {
                        ui.label("One or more addresses are invalid.");
                        if ui.button("Close").clicked() {
                            self.invalid_address_popup = false;
                        }
                    });
                }
            });
        });
    }
}

fn get_config() -> Result<Config, Error> {
    let data = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&data)?;

    Ok(config)
}

fn write_config(config: Config) {
    let json_data = serde_json::to_string_pretty(&config).expect("Failed to serialize to JSON");
    let mut file = File::create("config.json").expect("Failed to open file");
    file.write_all(json_data.as_bytes())
        .expect("Failed to write data");
}

fn begin_arbitrage(app: &mut App) {
    let config: Config = App::get_config();

    let transport: Http = match config.chain {
        Chain::Ethereum => {
            web3::transports::http::Http::new("https://rpc.api.moonbeam.network").unwrap()
        }
        Chain::Polygon => {
            web3::transports::http::Http::new("https://rpc.api.moonbeam.network").unwrap()
        }
        Chain::Binance => {
            web3::transports::http::Http::new("https://rpc.api.moonbeam.network").unwrap()
        }
    };
    let web3: Web3<Http> = web3::Web3::new(transport);

    let valid_bools: HashMap<String, bool> = check_valid_addresses(vec![
        config.contract_address,
        config.token_address_1,
        config.token_address_2,
        config.token_address_3,
        config.token_address_4,
    ]);

    if valid_bools.values().any(|&val| !val) {
        app.invalid_address_popup = true;
        return;
    }
}

// async fn arbitrage(
//     config: Config,
//     web3: Web3<Http>,
// ) -> web3::Result<()> {
//     loop {
//         let pair_addresses = get_pair_addresses(&web3, config).await?;
        
//         let (price_1, price_2) = tokio::try_join!(
//             get_price(&web3, pair_addresses),
//             get_price(&web3, &pair_address, config.token_2, config.token_1),
//         )?;
        
//         let fee_adjusted_price_1 = price_1 * (1.0 - config.transaction_fee);
//         let fee_adjusted_price_2 = price_2 * (1.0 - config.transaction_fee);
        
//         if fee_adjusted_price_1 > fee_adjusted_price_2 {
//             let tx1 = execute_trade(&web3, &pair_address, config.token_1, config.token_2, price_1).await?;
//             wait_for_transaction(&web3, tx1).await?;
//         } else if fee_adjusted_price_2 > fee_adjusted_price_1 {
//             let tx2 = execute_trade(&web3, &pair_address, config.token_2, config.token_1, price_2).await?;
//             wait_for_transaction(&web3, tx2).await?;
//         }

//         tokio::time::sleep(tokio::time::Duration::from_secs(config.refresh_interval)).await;
//     }
// }


// async fn get_pair_addresses(web3: &Web3<Http>, config: Config) -> web3::Result<Address> {
//     let factory_address: H160 = match H160::from_str(config.contract_address.as_str()) {
//         Ok(val) => val,
//         Err(_) => {}
//     };
//     let factory_contract = Contract::from_json(
//         web3.eth(),
//         factory_address,
//         include_bytes!("path_to_your/UniswapV2Factory.abi"),
//     );

//     let result: Address = factory_contract
//         .query(
//             "getPair",
//             (config.token_address_1, config.token_address_2),
//             None,
//             Options::default(),
//             None,
//         )
//         .await?;
//     Ok(result)
// }

// async fn get_price(web3: &Web3<Http>, pair_address: &Address) -> web3::Result<U256> {

//     let pair_contract = Contract::from_json(
//         web3.eth(),
//         pair_address,
//         include_bytes!("path_to_your/UniswapV2Pair.abi"),
//     )?;

//     let result: (U256, U256, u32) = pair_contract
//         .query("getReserves", (), None, Options::default(), None)
//         .await?;
//     let (reserve_eth, reserve_token, _timestamp) = result;

//     Ok(reserve_eth / reserve_token)
// }

// async fn execute_trade(
//     web3: &Web3<Http>,
//     eth_address: &Address,
//     token_address: &Address,
//     amount_in: U256,
// ) -> web3::Result<()> {
//     let router_address: Address = UNISWAP_ROUTER_ADDRESS.parse()?;
//     let router_contract = Contract::from_json(
//         web3.eth(),
//         router_address,
//         include_bytes!("path_to_your/UniswapV2Router02.abi"),
//     )?;

//     let deadline = U256::from(chrono::Utc::now().timestamp() + 300); // 5 minutes from now

//     let path = vec![eth_address.clone(), token_address.clone()];

//     // Define the transaction parameters
//     let tx = TransactionRequest {
//         from: "0xYourWalletAddressHere".parse()?,
//         to: Some(router_address),
//         gas: Some(GAS_LIMIT.into()),
//         gas_price: None, // Ideally you should estimate this
//         value: Some(amount_in),
//         data: Some(router_contract.encode(
//             "swapExactETHForTokens",
//             (
//                 U256::zero(),
//                 path,
//                 "0xYourWalletAddressHere".parse::<Address>()?,
//                 deadline,
//             ),
//         )?),
//         nonce: None,
//     };

//     // Send the transaction
//     let _tx_hash = web3.eth().send_transaction(tx).await?;

//     Ok(())
// }

fn check_valid_addresses(address_strs: Vec<String>) -> HashMap<String, bool> {
    let mut results = HashMap::new();

    for addr in address_strs.iter() {
        let is_valid = addr.parse::<Address>().is_ok();
        results.insert(addr.clone(), is_valid);
    }

    return results;
}

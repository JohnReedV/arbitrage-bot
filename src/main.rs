use eframe::egui;
use hex_literal::hex;
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
    types::{Address, TransactionRequest, H160, U256},
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
    gas_limit: f64,
    slippage_threshhold: f64,
    minimum_profit: f64,
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
            gas_limit: 0.0,
            slippage_threshhold: 0.0,
            minimum_profit: 0.0,
        }
    }
}

struct TempValues {
    temp_private_key_input: String,
    temp_token_address_input_1: String,
    temp_token_address_input_2: String,
    temp_token_address_input_3: String,
    temp_token_address_input_4: String,
    temp_selected_chain: Chain,
    temp_contract_address: String,
    temp_gas_limit: String,
    temp_slippage_threshhold: String,
    temp_minimum_profit: String,
}

impl TempValues {
    fn default() -> Self {
        TempValues {
            temp_private_key_input: String::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            temp_token_address_input_1: String::from("0x..."),
            temp_token_address_input_2: String::from("0x..."),
            temp_token_address_input_3: String::from("0x..."),
            temp_token_address_input_4: String::from("0x..."),
            temp_selected_chain: Chain::default(),
            temp_contract_address: String::from("0x..."),
            temp_gas_limit: String::from("0"),
            temp_slippage_threshhold: String::from("0"),
            temp_minimum_profit: String::from("0"),
        }
    }

    fn new(config: Config) -> Self {
        TempValues {
            temp_private_key_input: config.private_key,
            temp_token_address_input_1: config.token_address_1,
            temp_token_address_input_2: config.token_address_2,
            temp_token_address_input_3: config.token_address_3,
            temp_token_address_input_4: config.token_address_4,
            temp_selected_chain: config.chain,
            temp_contract_address: config.contract_address,
            temp_gas_limit: config.gas_limit.to_string(),
            temp_slippage_threshhold: config.slippage_threshhold.to_string(),
            temp_minimum_profit: config.minimum_profit.to_string(),
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
    show_gas_limit_error: bool,
    show_slippage_threshhold_error: bool,
    show_minimum_profit_error: bool,
    gas_limit: f64,
    slippage_threshhold: f64,
    minimum_profit: f64,
}

impl App {
    fn default() -> Self {
        App {
            selected_chain: Chain::Ethereum,
            private_key_input: String::new(),
            token_address_input_1: String::new(),
            token_address_input_2: String::new(),
            token_address_input_3: String::new(),
            token_address_input_4: String::new(),
            contract_address: String::new(),
            temp: TempValues::default(),
            account_text_dropped: false,
            invalid_address_popup: false,
            show_gas_limit_error: false,
            show_slippage_threshhold_error: false,
            show_minimum_profit_error: false,
            gas_limit: 0.0,
            slippage_threshhold: 0.0,
            minimum_profit: 0.0,
        }
    }

    fn new() -> Self {
        let current_config = get_config();

        match current_config {
            Ok(config) => {
                let config2 = config.clone();
                App {
                    selected_chain: config.chain,
                    private_key_input: config.private_key,
                    token_address_input_1: config.token_address_1,
                    token_address_input_2: config.token_address_2,
                    token_address_input_3: config.token_address_3,
                    token_address_input_4: config.token_address_4,
                    contract_address: config.contract_address,
                    temp: TempValues::new(config2),
                    account_text_dropped: false,
                    invalid_address_popup: false,
                    show_gas_limit_error: false,
                    show_slippage_threshhold_error: false,
                    show_minimum_profit_error: false,
                    gas_limit: config.gas_limit,
                    slippage_threshhold: config.slippage_threshhold,
                    minimum_profit: config.minimum_profit,
                }
            }
            Err(_) => return App::default(),
        }
    }

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
                gas_limit: config.gas_limit,
                slippage_threshhold: config.slippage_threshhold,
                minimum_profit: config.minimum_profit,
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

                        ui.label("Exchange Factory Contract Address :");
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
                            ui.label("Gas Limit: ");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.temp.temp_gas_limit)
                                    .desired_width(125.0),
                            );
                            ui.label("Slippage Threshhold: ");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.temp.temp_slippage_threshhold)
                                    .desired_width(125.0),
                            );
                            ui.label("Minimum Profit: ");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.temp.temp_minimum_profit)
                                    .desired_width(125.0),
                            );
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

                            if !self.temp.temp_gas_limit.is_empty() {
                                match self.temp.temp_gas_limit.parse::<f64>() {
                                    Ok(num) => {
                                        self.gas_limit = num;
                                    }
                                    Err(_) => {
                                        self.show_gas_limit_error = true;
                                    }
                                }
                            }
                            if !self.temp.temp_slippage_threshhold.is_empty() {
                                match self.temp.temp_slippage_threshhold.parse::<f64>() {
                                    Ok(num) => {
                                        self.slippage_threshhold = num;
                                    }
                                    Err(_) => {
                                        self.show_slippage_threshhold_error = true;
                                    }
                                }
                            }
                            if !self.temp.temp_minimum_profit.is_empty() {
                                match self.temp.temp_minimum_profit.parse::<f64>() {
                                    Ok(num) => {
                                        self.minimum_profit = num;
                                    }
                                    Err(_) => {
                                        self.show_minimum_profit_error = true;
                                    }
                                }
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
                                gas_limit: self.gas_limit.clone(),
                                slippage_threshhold: self.slippage_threshhold.clone(),
                                minimum_profit: self.minimum_profit.clone(),
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

                if self.show_gas_limit_error {
                    egui::Window::new("Invalid Gas Number").show(ctx, |ui| {
                        ui.label("Gas Limit must be a number");
                        if ui.button("Close").clicked() {
                            self.show_gas_limit_error = false;
                        }
                    });
                }
                if self.show_slippage_threshhold_error {
                    egui::Window::new("Invalid Slippage Number").show(ctx, |ui| {
                        ui.label("Slippage Threshhold must be a number");
                        if ui.button("Close").clicked() {
                            self.show_slippage_threshhold_error = false;
                        }
                    });
                }
                if self.show_minimum_profit_error {
                    egui::Window::new("Invalid Minimum Profit Number").show(ctx, |ui| {
                        ui.label("Minimum Profit must be a number");
                        if ui.button("Close").clicked() {
                            self.show_minimum_profit_error = false;
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
            web3::transports::http::Http::new(
                "https://mainnet.infura.io/v3/f679762894d44f4e88b1a37fbf30282b",
            )
            .unwrap() // https://ethereum.blockpi.network/v1/rpc/public
        }
        Chain::Polygon => {
            web3::transports::http::Http::new("https://polygon.blockpi.network/v1/rpc/public")
                .unwrap()
        }
        Chain::Binance => {
            web3::transports::http::Http::new("https://bsc-dataseed.bnbchain.org").unwrap()
        }
    };
    let web3: Web3<Http> = web3::Web3::new(transport);

    let valid_bools: HashMap<&String, bool> = check_valid_addresses(vec![
        &config.contract_address,
        &config.token_address_1,
        &config.token_address_2,
        &config.token_address_3,
        &config.token_address_4,
    ]);

    if valid_bools.values().any(|&val| !val) {
        app.invalid_address_popup = true;
        return;
    }

    tokio::spawn(async move {
        match arbitrage(config, web3).await {
            Ok(_) => println!("Arbitrage completed successfully"),
            Err(err) => eprintln!("Arbitrage failed: {}", err),
        }
    });
}

async fn arbitrage(config: Config, web3: Web3<Http>) -> web3::Result<()> {
    let price_pair_1 = get_price_from_dex(&web3, &config, 1).await?;
    let price_pair_2 = get_price_from_dex(&web3, &config, 2).await?;

    let spread = price_pair_1 - price_pair_2;

    if spread.abs() < config.slippage_threshhold {
        println!("Exiting: Slippage below threshold");
        return Ok(());
    }
    //MAKE IT PROFIT
    let gas_price = web3.eth().gas_price().await?;
    let total_gas_cost: f64 = gas_price.as_u64() as f64 * config.gas_limit;

    let potential_profit = spread - total_gas_cost;
    println!(
        "Price1: {:.18} Price2: {:.18} Gas: {} Profit: {}",
        price_pair_1, price_pair_2, total_gas_cost, potential_profit
    );
    if potential_profit < config.minimum_profit {
        println!("Exiting: Profit below threshold");
        return Ok(());
    }
    // let tx_hash_1;
    // let tx_hash_2;

    // if price_pair_1 > price_pair_2 {
    //     tx_hash_1 = execute_trade(&web3, &config, 1, 100.0, gas_price).await?;
    //     tx_hash_2 = execute_trade(&web3, &config, 2, 100.0, gas_price).await?;
    // } else if price_pair_2 > price_pair_1 {
    //     tx_hash_1 = execute_trade(&web3, &config, 1, -100.0, gas_price).await?;
    //     tx_hash_2 = execute_trade(&web3, &config, 2, -100.0, gas_price).await?;
    // } else {
    //     println!("Exiting: No arbitrage opportunity");
    //     return Ok(());
    // }

    // println!("Transaction 1 Hash: {}", tx_hash_1);
    // println!("Transaction 2 Hash: {}", tx_hash_2);

    Ok(())
}

async fn get_price_from_dex(web3: &Web3<Http>, config: &Config, pair_id: u8) -> web3::Result<f64> {
    let (token_a, token_b) = match pair_id {
        1 => (&config.token_address_1, &config.token_address_2),
        2 => (&config.token_address_3, &config.token_address_4),
        _ => return Err(web3::Error::InvalidResponse("Invalid pair_id".into())),
    };

    let factory_file = File::open("./factory_abi.json").unwrap();
    let factory_abi = web3::ethabi::Contract::load(factory_file).unwrap();

    let factory_contract = Contract::new(
        web3.eth(),
        H160::from_str(config.contract_address.as_str()).unwrap(),
        factory_abi,
    );

    let token_a_h160 = H160::from_str(token_a.as_str())
        .map_err(|e| web3::Error::InvalidResponse(format!("Failed to convert token_a: {:?}", e)))?;

    let token_b_h160 = H160::from_str(token_b.as_str())
        .map_err(|e| web3::Error::InvalidResponse(format!("Failed to convert token_b: {:?}", e)))?;

    let pool_address: Address = factory_contract
        .query(
            "getPool",
            (token_a_h160, token_b_h160, U256::from(3000)),
            None,
            Options::default(),
            None,
        )
        .await
        .map_err(|e| {
            web3::Error::InvalidResponse(format!("Factory contract query failed: {:?}", e))
        })?;

    let pool_file = File::open("./pool_abi.json").unwrap();
    let pool_abi = web3::ethabi::Contract::load(pool_file).unwrap();

    let pool_contract = Contract::new(web3.eth(), pool_address, pool_abi);

    let slot0: (U256, i64, U256, u32, u32, u32, bool) = pool_contract
        .query("slot0", (), None, Options::default(), None)
        .await
        .map_err(|e| {
            web3::Error::InvalidResponse(format!("Pool contract query failed: {:?}", e))
        })?;

    let decimals_token1 = fetch_decimals_of_token(web3, token_a_h160)
        .await
        .map_err(|e| {
            web3::Error::InvalidResponse(format!(
                "Failed to get decimals for token 1 in pair {}: {:?}",
                pair_id, e
            ))
        })?;

    let decimals_token2 = fetch_decimals_of_token(web3, token_b_h160)
        .await
        .map_err(|e| {
            web3::Error::InvalidResponse(format!(
                "Failed to get decimals for token 2 in pair {}: {:?}",
                pair_id, e
            ))
        })?;

    let sqrt_price_x96 = slot0.0;
    let sqrt_price_x96_f64 = u256_to_f64(sqrt_price_x96);
    let sqrt_price = sqrt_price_x96_f64 / (2_f64.powi(96));
    let price = sqrt_price * sqrt_price;
    let adj_price = price / 10_f64.powi(decimals_token1 as i32 - decimals_token2 as i32);

    Ok(adj_price)
}

// async fn execute_transaction() -> Result<(), Box<dyn std::error::Error>> {
//     let http = Http::new("http://localhost:8545")?;
//     let web3 = Web3::new(http);

//     let contract_address: Address = "0x...".parse()?;
//     let contract_abi = include_str!("../abi.json");

//     let contract = Contract::from_json(web3.eth(), contract_address, contract_abi.as_bytes())
//         .map_err(|e| format!("ABI Error: {}", e))?;

//     let some_arg1 = encode(&[U256::from(1).into()]);
//     let some_arg2 = encode(&[U256::from(2).into()]);
//     let args = vec![some_arg1, some_arg2];

//     let method_data = contract.abi.encode_function_call("yourMethodNameHere", &args)
//         .map_err(|e| format!("Encoding Error: {}", e))?;

//     let secret_key_bytes = hex::decode("your_private_key_here")?;
//     let secret_key = SecretKey::from_slice(&secret_key_bytes)?;

//     let tx = TransactionParameters {
//         to: Some(contract_address),
//         gas: U256::from(21000),
//         gas_price: Some(web3.eth().gas_price().await?),
//         data: Some(method_data.into()),
//         ..Default::default()
//     };

//     // The following assumes you are using an offline account
//     let account = web3::accounts::Account::Offline(secret_key, None);
//     let signed_tx = account.sign_transaction(tx).await?;
//     let tx_hash = web3.eth().send_raw_transaction(signed_tx.raw_transaction).await?;

//     println!("Transaction sent with hash: {:?}", tx_hash);

//     Ok(())
// }

fn check_valid_addresses(address_strs: Vec<&String>) -> HashMap<&String, bool> {
    let mut results = HashMap::new();

    for addr in address_strs {
        let is_valid = addr.parse::<Address>().is_ok();
        results.insert(addr, is_valid);
    }

    return results;
}
fn u256_to_f64(u: U256) -> f64 {
    let (upper, lower) = u.div_mod(U256::from(u64::MAX));
    (upper.as_u64() as f64) * ((u64::MAX as f64) + 1.0) + (lower.as_u64() as f64)
}

async fn fetch_decimals_of_token(web3: &Web3<Http>, token_address: H160) -> web3::Result<u8> {
    let token_abi_file = File::open("./erc20_abi.json").unwrap();
    let token_abi = web3::ethabi::Contract::load(token_abi_file).unwrap();
    let token_contract = Contract::new(web3.eth(), token_address, token_abi);
    let decimals: u8 = token_contract
        .query("decimals", (), None, Options::default(), None)
        .await
        .map_err(|e| {
            web3::Error::InvalidResponse(format!("Token contract query failed: {:?}", e))
        })?;

    Ok(decimals)
}

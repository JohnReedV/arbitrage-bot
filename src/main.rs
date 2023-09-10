use eframe::egui;
use hex::FromHex;
use reqwest::Client;
use secp256k1::{PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    fs::{self, File},
    io::{Error, Write},
    str::FromStr,
};
use tiny_keccak::{Hasher, Keccak};
use web3::{
    contract::{Contract, Options},
    ethabi::Token,
    signing::SecretKey,
    transports::Http,
    types::{Address, TransactionReceipt, H160, H256, U256},
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
    public_key: Address,
    token_address_master: String,
    token_address_1: String,
    token_address_2: String,
    gas_limit: u64,
    slippage_threshhold: f64,
    minimum_profit: f64,
    amount_to_trade: f64,
}

impl Config {
    fn default() -> Self {
        Config {
            chain: Chain::Ethereum,
            contract_address: String::new(),
            private_key: String::new(),
            public_key: Address::default(),
            token_address_master: String::new(),
            token_address_1: String::new(),
            token_address_2: String::new(),
            gas_limit: 0,
            slippage_threshhold: 0.0,
            minimum_profit: 0.0,
            amount_to_trade: 0.0,
        }
    }
}

struct TempValues {
    temp_private_key_input: String,
    temp_token_address_input_master: String,
    temp_token_address_input_1: String,
    temp_token_address_input_2: String,
    temp_selected_chain: Chain,
    temp_contract_address: String,
    temp_gas_limit: String,
    temp_slippage_threshhold: String,
    temp_minimum_profit: String,
    temp_amount_to_trade: String,
}

impl TempValues {
    fn default() -> Self {
        TempValues {
            temp_private_key_input: String::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            temp_token_address_input_master: String::from("0x..."),
            temp_token_address_input_1: String::from("0x..."),
            temp_token_address_input_2: String::from("0x..."),
            temp_selected_chain: Chain::default(),
            temp_contract_address: String::from("0x..."),
            temp_gas_limit: String::from("0"),
            temp_slippage_threshhold: String::from("0"),
            temp_minimum_profit: String::from("0"),
            temp_amount_to_trade: String::from("0.0"),
        }
    }

    fn new(config: Config) -> Self {
        TempValues {
            temp_private_key_input: config.private_key,
            temp_token_address_input_master: config.token_address_master,
            temp_token_address_input_1: config.token_address_1,
            temp_token_address_input_2: config.token_address_2,
            temp_selected_chain: config.chain,
            temp_contract_address: config.contract_address,
            temp_gas_limit: config.gas_limit.to_string(),
            temp_slippage_threshhold: config.slippage_threshhold.to_string(),
            temp_minimum_profit: config.minimum_profit.to_string(),
            temp_amount_to_trade: config.amount_to_trade.to_string(),
        }
    }
}

struct App {
    selected_chain: Chain,
    private_key_input: String,
    public_key: Address,
    token_address_input_master: String,
    token_address_input_1: String,
    token_address_input_2: String,
    contract_address: String,
    temp: TempValues,
    account_text_dropped: bool,
    invalid_address_popup: bool,
    show_gas_limit_error: bool,
    show_slippage_threshhold_error: bool,
    show_minimum_profit_error: bool,
    show_amount_to_trade_error: bool,
    invalid_private_key: bool,
    gas_limit: u64,
    slippage_threshhold: f64,
    minimum_profit: f64,
    amount_to_trade: f64,
}

impl App {
    fn default() -> Self {
        App {
            selected_chain: Chain::Ethereum,
            private_key_input: String::new(),
            public_key: Address::default(),
            token_address_input_master: String::new(),
            token_address_input_1: String::new(),
            token_address_input_2: String::new(),
            contract_address: String::new(),
            temp: TempValues::default(),
            account_text_dropped: false,
            invalid_address_popup: false,
            show_gas_limit_error: false,
            show_slippage_threshhold_error: false,
            show_minimum_profit_error: false,
            invalid_private_key: false,
            show_amount_to_trade_error: false,
            gas_limit: 0,
            slippage_threshhold: 0.0,
            minimum_profit: 0.0,
            amount_to_trade: 0.0,
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
                    public_key: config.public_key,
                    token_address_input_master: config.token_address_master,
                    token_address_input_1: config.token_address_1,
                    token_address_input_2: config.token_address_2,
                    contract_address: config.contract_address,
                    temp: TempValues::new(config2),
                    account_text_dropped: false,
                    invalid_address_popup: false,
                    show_gas_limit_error: false,
                    show_slippage_threshhold_error: false,
                    show_minimum_profit_error: false,
                    invalid_private_key: false,
                    show_amount_to_trade_error: false,
                    gas_limit: config.gas_limit,
                    slippage_threshhold: config.slippage_threshhold,
                    minimum_profit: config.minimum_profit,
                    amount_to_trade: config.amount_to_trade,
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
                public_key: config.public_key,
                token_address_master: config.token_address_master,
                token_address_1: config.token_address_1,
                token_address_2: config.token_address_2,
                gas_limit: config.gas_limit,
                slippage_threshhold: config.slippage_threshhold,
                minimum_profit: config.minimum_profit,
                amount_to_trade: config.amount_to_trade,
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

                        ui.label("Address of the token to arbitrage: ");
                        ui.text_edit_singleline(&mut self.temp.temp_token_address_input_master);

                        ui.label("Pairs to check Master Token price against");
                        ui.horizontal(|ui| {
                            ui.label("Token Address 1: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_1);
                            ui.label("Token Address 2: ");
                            ui.text_edit_singleline(&mut self.temp.temp_token_address_input_2);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Gas Limit: ");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.temp.temp_gas_limit)
                                    .desired_width(90.0),
                            );
                            ui.label("Slippage Threshhold: ");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.temp.temp_slippage_threshhold)
                                    .desired_width(90.0),
                            );
                            ui.label("Minimum Profit: ");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.temp.temp_minimum_profit)
                                    .desired_width(90.0),
                            );
                            ui.label("Amount to Trade:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.temp.temp_amount_to_trade)
                                    .desired_width(90.0),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Wallet Private Key: ");
                            ui.text_edit_singleline(&mut self.temp.temp_private_key_input);
                        });

                        if ui.button("Save").clicked() {
                            if !self.temp.temp_private_key_input.is_empty() {
                                self.private_key_input = self.temp.temp_private_key_input.clone();
                                let pub_key = priv_key_to_pub_key(&self.private_key_input);
                                if pub_key.is_ok() {
                                    self.public_key = pub_key.unwrap();
                                } else {
                                    self.invalid_private_key = true;
                                }
                            }
                            if !self.temp.temp_token_address_input_master.is_empty() {
                                self.token_address_input_master =
                                    self.temp.temp_token_address_input_master.clone();
                            }
                            if !self.temp.temp_token_address_input_1.is_empty() {
                                self.token_address_input_1 =
                                    self.temp.temp_token_address_input_1.clone();
                            }
                            if !self.temp.temp_token_address_input_2.is_empty() {
                                self.token_address_input_2 =
                                    self.temp.temp_token_address_input_2.clone();
                            }
                            if !self.temp.temp_contract_address.is_empty() {
                                self.contract_address = self.temp.temp_contract_address.clone();
                            }

                            if !self.temp.temp_gas_limit.is_empty() {
                                match self.temp.temp_gas_limit.parse::<u64>() {
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
                            if !self.temp.temp_amount_to_trade.is_empty() {
                                match self.temp.temp_amount_to_trade.parse::<f64>() {
                                    Ok(num) => {
                                        self.amount_to_trade = num;
                                    }
                                    Err(_) => {
                                        self.show_amount_to_trade_error = true;
                                    }
                                }
                            }

                            self.selected_chain = self.temp.temp_selected_chain;

                            let config = Config {
                                chain: self.selected_chain.clone(),
                                contract_address: self.contract_address.clone(),
                                private_key: self.private_key_input.clone(),
                                public_key: self.public_key.clone(),
                                token_address_master: self.token_address_input_master.clone(),
                                token_address_1: self.token_address_input_1.clone(),
                                token_address_2: self.token_address_input_2.clone(),
                                gas_limit: self.gas_limit.clone(),
                                slippage_threshhold: self.slippage_threshhold.clone(),
                                minimum_profit: self.minimum_profit.clone(),
                                amount_to_trade: self.amount_to_trade.clone(),
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
                if self.show_amount_to_trade_error {
                    egui::Window::new("Invalid Trade Amount Number").show(ctx, |ui| {
                        ui.label("Amount to Trade must be a number");
                        if ui.button("Close").clicked() {
                            self.show_amount_to_trade_error = false;
                        }
                    });
                }
                if self.invalid_private_key {
                    egui::Window::new("Invalid Private Key").show(ctx, |ui| {
                        ui.label("Provided Private Key is not valid");
                        if ui.button("Close").clicked() {
                            self.invalid_private_key = false;
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
        Chain::Ethereum => web3::transports::http::Http::new( "http://127.0.0.1:8545"
            /*"https://mainnet.infura.io/v3/f679762894d44f4e88b1a37fbf30282b"*/,
        )
        .unwrap(),
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
        &config.token_address_master,
        &config.token_address_1,
        &config.token_address_2,
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
    let (price_pair_1_f64, price_pair_1, pool_address_1) =
        get_price_and_pool_address(&web3, &config, 1).await?;
    let (price_pair_2_f64, price_pair_2, pool_address_2) =
        get_price_and_pool_address(&web3, &config, 2).await?;

    let profitable = is_arbitrage_profitable(
        &web3,
        pool_address_1,
        pool_address_2,
        price_pair_1,
        price_pair_2,
        price_pair_1_f64,
        price_pair_2_f64,
        &config,
    )
    .await;

    //let tx_hash;
    if profitable {
        //tx_hash = execute_trade(&web3, &config, pool_address_1, price_pair_1).await;
    } else {
        println!("Exiting: Profit below threshold");
        return Ok(());
    }

    // println!("Transaction 1 Hash: {:#?}", tx_hash);

    Ok(())
}

async fn is_arbitrage_profitable(
    web3: &Web3<Http>,
    pool_address_a: H160,
    pool_address_b: H160,
    price_pair_1: U256,
    price_pair_2: U256,
    price_a_eth: f64,
    price_b_eth: f64,
    config: &Config,
) -> bool {
    let mut cost_b_to_a: f64;
    let mut cost_a_to_b: f64;

    if let Ok(transaction_fee_a_to_b) =
        estimate_swap_fee(web3, &config, pool_address_a, price_pair_1).await
    {
        let fee_multiplier_a_to_b = 1.0 - (transaction_fee_a_to_b / 100.0);

        cost_a_to_b =
            price_a_eth * (price_b_eth * fee_multiplier_a_to_b).recip() * fee_multiplier_a_to_b;
        cost_a_to_b *= 1.0 + config.slippage_threshhold / 100.0;
        cost_a_to_b += config.gas_limit as f64;
    } else {
        return false;
    }

    if let Ok(transaction_fee_b_to_a) =
        estimate_swap_fee(web3, &config, pool_address_b, price_pair_2).await
    {
        let fee_multiplier_b_to_a = 1.0 - (transaction_fee_b_to_a / 100.0);
        cost_b_to_a =
            price_b_eth * (price_a_eth * fee_multiplier_b_to_a).recip() * fee_multiplier_b_to_a;

        cost_b_to_a *= 1.0 + config.slippage_threshhold / 100.0;
        cost_b_to_a += config.gas_limit as f64;
    } else {
        return false;
    }

    let profitable_a_to_b = cost_a_to_b < 1.0 && (1.0 - cost_a_to_b) >= config.minimum_profit;
    let profitable_b_to_a = cost_b_to_a < 1.0 && (1.0 - cost_b_to_a) >= config.minimum_profit;

    profitable_a_to_b || profitable_b_to_a
}

async fn execute_trade(
    web3: &Web3<Http>,
    config: &Config,
    pool_address: Address,
    price: f64,
) -> Result<H256, web3::Error> {
    let prvk = SecretKey::from_str(&config.private_key).unwrap();

    let pool_file = File::open("./pool_abi.json").unwrap();
    let pool_abi = web3::ethabi::Contract::load(pool_file).unwrap();
    let pool_contract = Contract::new(web3.eth(), pool_address, pool_abi);

    let amount: U256 = f64_to_u256(config.amount_to_trade);

    let result: Result<H256, web3::Error> = pool_contract
        .signed_call(
            "swap",
            (
                Token::Address(config.public_key),
                Token::Bool(true),
                Token::Int(amount),
                Token::Uint(calculate_sqrt_price_limit(price).into()),
                Token::Bytes(Vec::new()),
            ),
            web3::contract::Options::default(),
            &prvk,
        )
        .await
        .map_err(|e| web3::Error::InvalidResponse(format!("Failed to send: {}", e)));

    match &result {
        Ok(tx_hash) => {
            let bytes = tx_hash.0;
            println!("Transaction hash as bytes: {:?}", bytes);

            let hex = tx_hash.to_string();
            println!("Transaction hash as hex: {}", hex);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    return result;
}

async fn estimate_swap_fee(
    web3: &Web3<Http>,
    config: &Config,
    pool_address: Address,
    current_sqrt_price: U256,
) -> Result<f64, web3::Error> {
    let prvk = SecretKey::from_str(&config.private_key).unwrap();
    let pool_file = File::open("./pool_abi.json").unwrap();
    let pool_abi = web3::ethabi::Contract::load(pool_file).unwrap();
    let pool_contract = Contract::new(web3.eth(), pool_address, pool_abi);

    let min_sqrt_ratio: U256 = U256::from_dec_str("4295128739").unwrap();
    if current_sqrt_price <= min_sqrt_ratio {
        return Err(web3::Error::InvalidResponse("Invalid sqrtPriceX96".into()));
    }
    let delta: U256 = U256::from(10);
    let sqrt_price_limit_x96: U256 = current_sqrt_price.saturating_sub(delta);

    let mut options = web3::contract::Options::default();
    options.gas = Some(config.gas_limit.into());

    let approval = approve_erc20(&web3, &config).await;

    match approval {
        Ok(_) => {
            let result: Result<TransactionReceipt, web3::Error> = pool_contract
                .signed_call_with_confirmations(
                    "swap",
                    (
                        Token::Address(config.public_key),
                        Token::Bool(true),
                        Token::Int(f64_to_u256(config.amount_to_trade)),
                        Token::Uint(sqrt_price_limit_x96),
                        Token::Bytes(Vec::new()),
                    ),
                    options,
                    1,
                    &prvk,
                )
                .await
                .map_err(|e| web3::Error::InvalidResponse(format!("Failed to send: {}", e)));

            match &result {
                Ok(tx_hash) => {
                    println!("Transaction hash as hex: {:#?}", tx_hash);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Err(why) => {
            web3::Error::InvalidResponse(format!("Failed to approve: {}", why));
        }
    }

    return Ok(0.0);
}

async fn approve_erc20(web3: &Web3<Http>, config: &Config) -> web3::Result<()> {
    let prvk = SecretKey::from_str(&config.private_key).unwrap();
    let erc20_file = File::open("./erc20_abi.json").unwrap();
    let erc20_abi = web3::ethabi::Contract::load(erc20_file).unwrap();

    let contract1 = Contract::new(
        web3.eth(),
        H160::from_str(config.token_address_1.as_str()).unwrap(),
        erc20_abi.clone(),
    );
    // let contract2 = Contract::new(
    //     web3.eth(),
    //     H160::from_str(config.token_address_2.as_str()).unwrap(),
    //     erc20_abi.clone(),
    // );

    let approve_params = (
        Token::Address(config.public_key),
        Token::Uint(f64_to_u256(config.amount_to_trade)),
    );

    let tx_hash1 = contract1
        .signed_call_with_confirmations(
            "approve",
            approve_params.clone(),
            Default::default(),
            1,
            &prvk,
        )
        .await
        .map_err(|e| web3::Error::InvalidResponse(format!("Approval 1 failed: {}", e)));

    // let tx_hash2 = contract2
    //     .call(
    //         "approve",
    //         approve_params.clone(),
    //         config.public_key,
    //         Default::default(),
    //     )
    //     .await
    //     .map_err(|e| web3::Error::InvalidResponse(format!("Approval 2 failed: {}", e)));

    println!("Transaction 1 Hash: {:?}", tx_hash1);
    //println!("Transaction 2 Hash: {:?}", tx_hash2);

    Ok(())
}

async fn get_price_and_pool_address(
    web3: &Web3<Http>,
    config: &Config,
    pair_id: u8,
) -> web3::Result<(f64, U256, Address)> {
    let (token_a, token_b) = match pair_id {
        1 => (&config.token_address_master, &config.token_address_1),
        2 => (&config.token_address_master, &config.token_address_2),
        _ => return Err(web3::Error::InvalidResponse("Invalid pair_id".into())),
    };

    let factory_file = File::open("./factory_abi.json").unwrap();
    let factory_abi = web3::ethabi::Contract::load(factory_file).unwrap();

    let factory_contract = Contract::new(
        web3.eth(),
        H160::from_str(config.contract_address.as_str()).unwrap(),
        factory_abi,
    );

    let token_a_h160 = token_string_to_h160(token_a)?;
    let token_b_h160 = token_string_to_h160(token_b)?;

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
    if pool_address.is_zero() {
        return Err(web3::Error::InvalidResponse(format!(
            "No pair address for token pair: {}",
            pair_id
        )));
    }

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

    Ok((adj_price, sqrt_price_x96, pool_address))
}

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

fn token_string_to_h160(token: &String) -> web3::Result<H160> {
    return H160::from_str(token.as_str())
        .map_err(|e| web3::Error::InvalidResponse(format!("Failed to convert token_a: {:?}", e)));
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

pub fn calculate_sqrt_price_limit(price: f64) -> u128 {
    let sqrt_price = price.sqrt();
    let sqrt_price_fixed_point: u128 = (sqrt_price * (1u64 << 48) as f64) as u128;
    return sqrt_price_fixed_point << 48;
}

pub fn priv_key_to_pub_key(private_key: &String) -> Result<Address, &'static str> {
    let secp: Secp256k1<secp256k1::All> = Secp256k1::new();

    let private_key_bytes: Vec<u8> = match Vec::from_hex(private_key) {
        Ok(bytes) => bytes,
        Err(_) => return Err("Invalid hex string"),
    };

    let secret_key: SecretKey = match SecretKey::from_slice(&private_key_bytes) {
        Ok(key) => key,
        Err(_) => return Err("Invalid private key"),
    };

    let public_key: PublicKey = PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_bytes: [u8; 65] = public_key.serialize_uncompressed();
    let public_key_bytes: &[u8] = &public_key_bytes[1..65];

    let mut hasher: Keccak = Keccak::v256();
    hasher.update(public_key_bytes);
    let mut result: [u8; 32] = [0u8; 32];
    hasher.finalize(&mut result);

    let address_bytes: &[u8] = &result[12..];
    let address: H160 = Address::from_slice(address_bytes);

    Ok(address)
}

fn f64_to_u256(val: f64) -> U256 {
    let val_str = val.to_string();
    let parts: Vec<&str> = val_str.split('.').collect();
    let int_part: U256 = parts[0].parse::<U256>().unwrap();
    let decimal_part_str: String;

    if parts.len() > 1 {
        decimal_part_str = format!("0.{}", parts[1]);
    } else {
        decimal_part_str = "0.0".to_string();
    }

    let decimal_part_f64: f64 = decimal_part_str.parse().unwrap();
    let decimal_part_u256: U256 = ((decimal_part_f64 * 1e18).round() as u64).into();

    int_part * U256::exp10(18) + decimal_part_u256
}

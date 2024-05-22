use axum::{extract::Json, routing::get, routing::post, Router};
use dotenv::dotenv;
use fuels::accounts::wallet::Wallet;
use fuels::{crypto::SecretKey, prelude::*};
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
struct TransferRes {
    success: bool,
    tx_id: String,
    explorer_url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct TransferPost {
    address: String,
    network: String,
    amount: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Setting {
    amount: String,
    frequency: String,
    alarm: String,
    chain_type: String,
}

#[warn(unused_must_use)]
async fn transfer(data: Json<TransferPost>) -> Json<TransferRes> {
    // Create a provider pointing to the testnet.
    // This example will not work as the testnet does not support the new version of fuel-core
    // yet

    println!("data: {:#?}", data);
    let key = env::var("KEY").expect("KEY 未设置");
    let provider = Provider::connect(&data.network).await.unwrap();

    // Setup a private key
    let secret = SecretKey::from_str(&key).unwrap();

    // Create the wallet
    let wallet = WalletUnlocked::new_from_private_key(secret, Some(provider));

    // Get the wallet address. Used later with the faucet
    println!("{}", wallet.address().to_string());

    let url = env::var("SETTING_URL").expect("SETTING_URL 未设置");
    let response = reqwest::get(&url).await.unwrap();

    let setting: Setting = response.json().await.unwrap();
    println!("setting: {:#?}", setting);

    let asset_id: AssetId = BASE_ASSET_ID;
    let balance: u64 = wallet.get_asset_balance(&asset_id).await.unwrap();

    println!("balance: {}, asset_id: {} ", balance, asset_id);

    // const NUM_ASSETS: u64 = 0;
    let amount: u64 = data.amount.to_owned().parse().unwrap();
    // let amount: u64 = 10000000000;

    // const NUM_COINS: u64 = 1;
    // let (coins, _) = setup_multiple_assets_coins(wallet.address(), NUM_ASSETS, NUM_COINS, AMOUNT);

    let receiver = Bech32Address::from_str(&data.address).unwrap();

    // let (_tx_id, _receipts) = wallet
    //     .transfer(&receiver, amount, asset_id, TxPolicies::default())
    //     .await
    //     .unwrap();

    let result = wallet
        .transfer(&receiver, amount, asset_id, TxPolicies::default())
        .await;

    match result {
        Ok((_tx_id, _receipts)) => {
            // 处理成功的结果
            println!("Transaction successful: {:?}", _tx_id);
            Json(TransferRes {
                success: true,
                tx_id: _tx_id.to_string(),
                explorer_url: explorer_url(&_tx_id.to_string()),
            })
        }
        Err(e) => {
            // 处理错误
            eprintln!("Transaction failed: {:?}", e);
            Json(TransferRes {
                success: false,
                tx_id: String::new(),
                explorer_url: String::new(),
            })
        }
    }
}

fn explorer_url(tx_id: &str) -> String {
    let base_url = "https://app.fuel.network/tx/0x";
    let path = "/simple";
    format!("{}{}{}", base_url, tx_id, path)
}

#[tokio::main]
async fn main() {
    // 加载 .env 文件
    dotenv().ok();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/fuel/request", post(transfer));

    let port = env::var("PORT").unwrap_or_else(|_| "6004".to_string());
    let addr = format!("0.0.0.0:{}", port);
    // run it with hyper on localhost:3000
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

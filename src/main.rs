use axum::{extract::Json, routing::get, routing::post, Router};
use dotenv::dotenv;
use fuels::accounts::wallet::Wallet;
use fuels::{crypto::SecretKey, prelude::*};
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
struct TransferRes {
    from: String,
    to: String,
    amount: u64,
    asset_id: String,
    tx_id: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct TransferPost {
  receiver: String,
  amount: u64,
}

#[warn(unused_must_use)]
async fn transfer(data: Json<TransferPost>) -> Json<TransferRes> {
    // Create a provider pointing to the testnet.
    // This example will not work as the testnet does not support the new version of fuel-core
    // yet

    println!("data: {:#?}", data);
    let key = env::var("KEY").expect("KEY 未设置");
    let provider = Provider::connect("beta-5.fuel.network").await.unwrap();

    // Setup a private key
    let secret = SecretKey::from_str(&key).unwrap();

    // Create the wallet
    let wallet = WalletUnlocked::new_from_private_key(secret, Some(provider));

    // Get the wallet address. Used later with the faucet
    println!("{}", wallet.address().to_string());

    let asset_id: AssetId = BASE_ASSET_ID;
    let balance: u64 = wallet.get_asset_balance(&asset_id).await.unwrap();

    println!("balance: {}, asset_id: {} ", balance, asset_id);

    // const NUM_ASSETS: u64 = 0;
    let amount: u64 = data.amount;
    // const NUM_COINS: u64 = 1;
    // let (coins, _) = setup_multiple_assets_coins(wallet.address(), NUM_ASSETS, NUM_COINS, AMOUNT);

    let receiver =
        Bech32Address::from_str(&data.receiver)
            .unwrap();

    let (_tx_id, _receipts) = wallet
        .transfer(&receiver, amount, asset_id, TxPolicies::default())
        .await
        .unwrap();

    println!("_tx_id: {}, _receipts: {:#?} ", _tx_id, _receipts);

    Json(TransferRes {
        from: wallet.address().to_string(),
        to: receiver.to_string(),
        amount: amount,
        asset_id: asset_id.to_string(),
        tx_id: _tx_id.to_string(),
    })
}

#[tokio::main]
async fn main() {
    // 加载 .env 文件
    dotenv().ok();

    // setup().await;

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/transfer", post(transfer));

    // run it with hyper on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

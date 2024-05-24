use axum::http::StatusCode;
use axum::response::Json as AxumJson;
use axum::{extract::Json, response::IntoResponse, routing::get, routing::post, Router};
use dotenv::dotenv;
use fuels::{crypto::SecretKey, prelude::*};
use serde::{Deserialize, Serialize};
use std::env;
use std::num::ParseIntError;
use std::result::Result as StdResult;
use std::str::FromStr;
use thiserror::Error;

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
struct TransferErrorRes {
    success: bool,
    message: String,
}

#[derive(Debug, Error)]
enum TransferError {
    #[error("Network connection error: {0}")]
    NetworkError(String),
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Failed to get asset balance: {0}")]
    GetBalanceError(String),
    #[error("Invalid amount format: {0}")]
    InvalidAmountFormat(String),
    #[error("Invalid receiver address: {0}")]
    InvalidReceiverAddress(String),
    #[error("Transaction failed: {0}")]
    TransactionError(String),
}

impl IntoResponse for TransferError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        let error_res = TransferErrorRes {
            success: false,
            message,
        };
        let json = AxumJson(error_res);
        (StatusCode::INTERNAL_SERVER_ERROR, json).into_response()
    }
}

#[warn(unused_must_use)]
async fn transfer(data: Json<TransferPost>) -> StdResult<Json<TransferRes>, TransferError> {
    // Create a provider pointing to the testnet.
    println!("data: {:#?}", data);
    let key = env::var("KEY").expect("KEY 未设置");

    // 网络可能连接失败
    let provider = Provider::connect(&data.network)
        .await
        .map_err(|err| TransferError::NetworkError(err.to_string()))?;

    // Setup a private key
    let secret = SecretKey::from_str(&key)
        .map_err(|err| TransferError::InvalidPrivateKey(err.to_string()))?;

    // Create the wallet
    let wallet = WalletUnlocked::new_from_private_key(secret, Some(provider));

    // Get the wallet address. Used later with the faucet
    println!("{}", wallet.address().to_string());

    let asset_id: AssetId = BASE_ASSET_ID;
    let balance = wallet
        .get_asset_balance(&asset_id)
        .await
        .map_err(|err| TransferError::GetBalanceError(err.to_string()))?;

    println!("balance: {}, asset_id: {} ", balance, asset_id);

    let amount: u64 = data
        .amount
        .to_owned()
        .parse()
        .map_err(|err: ParseIntError| TransferError::InvalidAmountFormat(err.to_string()))?;

    let receiver = Bech32Address::from_str(&data.address)
        .map_err(|err| TransferError::InvalidReceiverAddress(err.to_string()))?;
    // Send the transaction
    let (_tx_id, _receipts) = wallet
        .transfer(&receiver, amount, asset_id, TxPolicies::default())
        .await
        .map_err(|err: Error| TransferError::TransactionError(err.to_string()))?;

    println!("Transaction successful: {:?}", _tx_id);

    Ok(Json(TransferRes {
        success: true,
        tx_id: _tx_id.to_string(),
        explorer_url: explorer_url(&_tx_id.to_string()),
    }))

    // match result {
    //     Ok((_tx_id, _receipts)) => {
    //         // 处理成功的结果
    //         println!("Transaction successful: {:?}", _tx_id);
    //         Ok(Json(TransferRes {
    //             success: true,
    //             tx_id: _tx_id.to_string(),
    //             explorer_url: explorer_url(&_tx_id.to_string()),
    //         }))
    //     }
    //     Err(e) => {
    //         // 处理错误
    //         eprintln!("Transaction failed: {:?}", e);
    //         Err(TransferError::TransactionError(e.to_string()))
    //     }
    // }
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

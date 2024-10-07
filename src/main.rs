use clap::Parser;
use regex::Regex;

// using our cosmos-rs library
extern crate cosmos;
use cosmos::*;

// Error handling
use anyhow::{anyhow, Result};

// Logging
extern crate log;
use log::info;

use std::str::FromStr;
use std::result::Result::Ok;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    operation: String,

    /// Number of times to greet
    address: String,
}

#[derive(Debug)]
struct Txn {
    address: String,
    amount: u32,
    denom: String,
}

struct SenderInfo {
    wallet: Wallet,
    address: Address
}

#[tokio::main(flavor = "current_thread")]
async fn main() {

    // Enable Logging
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    match parse_args().await {
        Ok(txn) => { 
            let _ = main_execution(txn).await;
        }
        Err(e) => println!("Error: {}", e),
    }
}

async fn parse_args() -> Result<Txn, anyhow::Error> {
    let args = Args::parse();

    // (?<amount>\d+(?:\.*\d+)*)(?<denom>[a-zA-Z]*) -> This one admits float
    let re = Regex::new(r"(?<amount>\d+)(?<denom>[a-zA-Z]*)").unwrap();

    let Some(caps) = re.captures(&args.operation) else {
        return Err(anyhow!("Invalid Args provided"));
    };

    if caps.len() < 3 || caps.len() > 3{
        return Err(anyhow!("Unexpected amount fo matches"));
    }

    info!("Recipient Address: {}", args.address);
    info!("Amount: {}", &caps["amount"]);
    info!("Denom: {}", &caps["denom"]);

    let address: String = args.address;
    let amount: u32 = caps["amount"].parse::<u32>().unwrap();
    let denom: String = caps["denom"].to_string();
    
    Ok(Txn { address, amount, denom })
}

async fn connect_to_chain() -> Result<Cosmos, anyhow::Error> {
    // Connect to the blockchain
    let cosmos_handle = CosmosNetwork::OsmosisTestnet.connect().await?;
    // log::info!("Successfully connected to chain");
    Ok(cosmos_handle)
}

async fn get_coin_balance(chain: &Cosmos, address: Address, denom: &str) -> Result<String, anyhow::Error> {
    let balances: Vec<Coin> = cosmos::Cosmos::all_balances(&chain, address).await?;
    let mut amount = "0";

    balances.iter().for_each(|balance| {
        if balance.denom == denom { 
            amount = &balance.amount;
        }
    });

    Ok(amount.to_string())
}

async fn get_sender_info() -> Result<SenderInfo, anyhow::Error> {
    const PHRASE: &str =
    "again ready search face detail violin gesture pluck tuition dinner wealth debate exclude okay wait raven hawk gold dream myself bullet pitch barely tortoise";

    let seed_phrase = SeedPhrase::from_str(PHRASE).unwrap();

    let wallet = seed_phrase
                        .with_hrp(AddressHrp::from_static("osmo"))
                        .unwrap();

    // My Wallet: osmo1vmlnscnqv2wevdxczxl53dz0kmdwqdw4glhvdw
    // let sender_address_str = wallet.get_address_string();
    let sender_address: Address = wallet.get_address();

    Ok(SenderInfo { wallet: wallet, address: sender_address })
}

async fn get_sender_balance(chain: &Cosmos, sender_address: &Address, txn: &Txn) -> Result<(), anyhow::Error> {
    let amount = get_coin_balance(chain, *sender_address, &txn.denom).await?;
    info!("Sender's balance: {} {}", amount, &txn.denom);
    Ok(())
}

async fn get_recipient_address(txn: &Txn) -> Result<Address, anyhow::Error> {
    let recipient_address_str: std::result::Result<Address, error::AddressError> = Address::from_str(&txn.address);
    let recipient_address = recipient_address_str?.get_address();
    Ok(recipient_address)
}

async fn get_recipient_balance(chain: &Cosmos, recipient_address: &Address, txn: &Txn) -> Result<(), anyhow::Error> {
    let amount = get_coin_balance(chain, *recipient_address, &txn.denom).await?;
    info!("Recipient's balance: {} {}", amount, &txn.denom);
    Ok(())
}

async fn execute_txn(chain: &Cosmos, txn: &Txn, sender_wallet: &Wallet, recipient_address: &Address) -> Result<(), anyhow::Error> {
    let amount_to_send: Vec<Coin> = vec![ Coin{denom: txn.denom.to_owned(), amount: txn.amount.to_string()} ];
    sender_wallet.send_coins(&chain, *recipient_address, amount_to_send).await?;
    Ok(())
}

async fn main_execution(txn: Txn) -> Result<(), anyhow::Error> {
    // Connect to Cosmos chain
    let chain: Cosmos = connect_to_chain().await?;

    // Get Sender and Recipient info
    let sender_info = get_sender_info().await?;
    let recipient_address = get_recipient_address(&txn).await?;

    // Before operation
    info!("--- BALANCES BEFORE TRANSACTION ---");
    get_sender_balance(&chain, &sender_info.address, &txn).await?;      // Get Sender balances and log them
    get_recipient_balance(&chain, &recipient_address, &txn).await?;                     // Get Recipient balances and log them

    // Execute transaction
    info!("--- EXECUTING TRANSACTION ---");
    execute_txn(&chain, &txn, &sender_info.wallet, &recipient_address).await?;
    info!("--- COMPLETED TRANSACTION ---");
    // After operation
    info!("--- BALANCES AFTER TRANSACTION ---");
    get_sender_balance(&chain, &sender_info.address, &txn).await?;      // Get Sender balances and log them
    get_recipient_balance(&chain, &recipient_address, &txn).await?;                     // Get Recipient balances and log them

    Ok(())
}
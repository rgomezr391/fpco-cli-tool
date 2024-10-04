use clap::Parser;
use regex::Regex;

// using our cosmos-rs library
extern crate cosmos;
use cosmos::*;


// Error handling
use anyhow::{anyhow, Context, Ok, Result};

// Logging
extern crate log;
use log::{debug, error, log_enabled, info, Level};

use std::str::FromStr;
use crate::proto::cosmos::auth::v1beta1::BaseAccount;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    operation: String,

    /// Number of times to greet
    address: String,
}

#[derive(Debug)]
struct Txn{
    address: String,
    amount: u32,
    denom: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();

    info!("Operation: {}", args.operation);
    info!("Address: {}", args.address);

    let re = Regex::new(r"(?<amount>\d*)(?<denom>\w*)").unwrap();

    let Some(caps) = re.captures(&args.operation) else {
        info!("no match!");
        return;
    };
    info!("Amount: {}", &caps["amount"]);
    info!("Denom: {}", &caps["denom"]);

    let address: String = args.address;
    let amount: u32 = caps["amount"].parse::<u32>().unwrap();
    let denom: String = caps["denom"].to_string();


    let txn = Txn { address, amount, denom };

    another_fn(txn).await;
}

async fn another_fn(txn: Txn) -> Result<(), anyhow::Error> {
    // Execute the transaction

    let chain: Cosmos = connect_to_chain().await?;

    log::info!("Successfully connected to chain");

    /////////////////////////////////////////////////
    /// 
    /// Get my data

    const PHRASE: &str =
            "again ready search face detail violin gesture pluck tuition dinner wealth debate exclude okay wait raven hawk gold dream myself bullet pitch barely tortoise";

    let seed_phrase = SeedPhrase::from_str(PHRASE).unwrap();

    let wallet = seed_phrase
                            .with_hrp(AddressHrp::from_static("osmo"))
                            .unwrap();

    let sender_address_str = wallet.get_address_string();
    info!("Raul's Wallet address is {}", sender_address_str);
    // My Wallet: osmo1vmlnscnqv2wevdxczxl53dz0kmdwqdw4glhvdw

    let sender_address = wallet.get_address();
    let balances = cosmos::Cosmos::all_balances(&chain, sender_address).await?;

    // Iterate over balances, for each
    balances.iter().for_each(|balance| {
        // Show and record info
        if balance.denom == "uosmo" { 
            info!("Raul's UOSMO balance: {} {}", balance.amount, balance.denom);
        }
    });

    /////////////////////////////////////////////////

    let receiver_address_str: std::result::Result<Address, error::AddressError> = Address::from_str(&txn.address);
    let receiver_address = receiver_address_str?.get_address();

    //let base_account: BaseAccount = chain.get_base_account(real_address).await?;

    // Get all balances
    let balances = cosmos::Cosmos::all_balances(&chain, receiver_address).await?;

    // Iterate over balances, for each
    balances.iter().for_each(|balance| {
        // Show and record info
        if balance.denom == "uosmo" { 
            info!("Receiver's UOSMO balance: {} {}", balance.amount, balance.denom);
        }
    });

    /////////////////////////////////////////////////

    let amount_to_send: Vec<Coin> = vec![Coin{denom: txn.denom.to_owned(), amount: txn.amount.to_string()}];

    wallet.send_coins(&chain, receiver_address, amount_to_send).await?;
    
    Ok(())

    // match connect_to_chain().await {
    //     Ok(_)=>{
    //         log::info!("Successfully connected to chain");
    //         Ok(())
    //     }
    //     Err(e)=>{
    //         error!("{}", e);
    //         return Err(e);
    //     }
    // }
}

async fn connect_to_chain() -> Result<Cosmos, anyhow::Error> {
    // Connect to the blockchain
    let cosmos_handle = CosmosNetwork::OsmosisTestnet.connect().await?;
    
    Ok(cosmos_handle)
}
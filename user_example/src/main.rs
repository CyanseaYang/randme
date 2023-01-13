// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use sui_sdk::{
    SuiClient,
    rpc_types::{SuiEvent, SuiEventFilter},
    json::SuiJsonValue,
};
use sui_types::{
    base_types::ObjectID,
    event::EventType,
    messages::{Transaction, ExecuteTransactionRequestType},
    object::Owner,
    intent::Intent,
};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_adapter::execution_mode;
use futures::StreamExt;
use std::str::FromStr;

const PACKAGEID: &str = "0x64aa4d601da1ed80fcb94d982fd438c9a448a417";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let package_id = ObjectID::from_hex_literal(PACKAGEID).unwrap();
    let keystore_path = match dirs::home_dir() {
        Some(v) => v.join(".sui").join("sui_config").join("sui.keystore"),
        None => panic!("cannot obtain home directory path"),
    };
    let keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);
    let signer = keystore.addresses()[0];
    
    let sui = SuiClient::new("https://fullnode.devnet.sui.io:443", Some("wss://fullnode.devnet.sui.io:443"), None).await?;
    let filters = vec![
        SuiEventFilter::Module("vrf".to_string()),
        SuiEventFilter::EventType(EventType::NewObject),
    ];
    let mut subscribe_all = sui
        .event_api()
        .subscribe_event(SuiEventFilter::All(filters))
        .await?;
    loop {
        if let Some(Ok(envelope)) = subscribe_all.next().await {
            match envelope.event {
                SuiEvent::NewObject { 
                    package_id: _, 
                    transaction_module: _,
                    sender: _,
                    recipient,
                    object_type: _,
                    object_id,
                    version: _,
                } => {
                    match recipient {
                        Owner::AddressOwner(address) => {
                            if &address == &signer {
                                //println!("build transaction...");
                                let fulfill_call = sui
                                    .transaction_builder()
                                    .move_call::<execution_mode::Normal>(
                                        signer,
                                        package_id,
                                        "client",
                                        "fulfill_randme",
                                        vec![],
                                        vec![
                                            SuiJsonValue::from_str(&object_id.to_hex_literal())?,
                                        ],
                                        None,
                                        1000,
                                    )
                                    .await?;
                                let signature = keystore
                                    .sign_secure(&signer, &fulfill_call, Intent::default())?;
                                let response = sui
                                    .quorum_driver()
                                    .execute_transaction(
                                        Transaction::from_data(fulfill_call, Intent::default(), signature).verify()?,
                                        Some(ExecuteTransactionRequestType::WaitForLocalExecution),
                                    )
                                    .await?;
                                println!("tx_response: {:?}", response);
                            }
                        }
                        _ => {}
                    }    
                }
                _ => {}
            }
        }
    }
}

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use sui_sdk::{
    SuiClient,
    rpc_types::{SuiEvent, SuiEventFilter, SuiMoveStruct, SuiMoveValue},
    json::SuiJsonValue,
};
use sui_types::{
    base_types::{ObjectID},
    messages::{Transaction, ExecuteTransactionRequestType},
    intent::Intent,
};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use futures::StreamExt;
use std::str::FromStr;

use fastcrypto::bls12381::min_sig::*;
use fastcrypto::{traits::{KeyPair, Signer}};
use rand::thread_rng;

const PACKAGEID: &str = "0x1fe19419c6aeb4201fea65c2487857eb2621011f";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let sui = SuiClient::new("https://fullnode.devnet.sui.io:443", Some("wss://fullnode.devnet.sui.io:443"), None).await?;
    let package_id = ObjectID::from_hex_literal(PACKAGEID).unwrap();
    let keystore_path = match dirs::home_dir() {
        Some(v) => v.join(".sui").join("sui_config").join("sui.keystore"),
        None => panic!("cannot obtain home directory path"),
    };
    let keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);
    let signer = keystore.addresses()[0];
    let bls_kp = BLS12381KeyPair::generate(&mut thread_rng());
    println!("bls public key: {}", &bls_kp.public());

    let filters = vec![
        SuiEventFilter::MoveEventType(format!("{}::vrf::RequestEvent", PACKAGEID)),
    ];
    let mut subscribe_all = sui
        .event_api()
        .subscribe_event(SuiEventFilter::All(filters))
        .await?;
    loop {
        if let Some(Ok(envelope)) = subscribe_all.next().await {
            println!("{:?}", envelope);
            match envelope.event {
                SuiEvent::MoveEvent { 
                    package_id: _, 
                    transaction_module: _,
                    sender: _,
                    type_: _,
                    fields,
                    bcs: _,
                } => {
                    if let Some(move_struct) = fields {
                        match move_struct {
                            SuiMoveStruct::WithFields(fields) | SuiMoveStruct::WithTypes { type_: _, fields } => {
                                if let Some(SuiMoveValue::String(seed)) = fields.get("seed") {
                                    if let Ok(seed) = seed.parse::<u64>() {
                                        if let Some(SuiMoveValue::Address(consumer)) = fields.get("consumer") {
                                            let mut msg: Vec<u8> = Vec::new();
                                            msg.extend_from_slice(&bcs::to_bytes(&seed).unwrap());
                                            msg.extend_from_slice(&bcs::to_bytes(&consumer).unwrap());
                                            let bls_sig = bls_kp.sign(&msg[..]);
                                            
                                            let verify_call = sui
                                                .transaction_builder()
                                                .move_call(
                                                    signer,
                                                    package_id,
                                                    "vrf",
                                                    "verify",
                                                    vec![],
                                                    vec![
                                                        SuiJsonValue::new(bls_sig.as_ref().into())?,
                                                        SuiJsonValue::new(bls_kp.public().as_ref().into())?,
                                                        SuiJsonValue::from_str(&seed.to_string())?,
                                                        SuiJsonValue::from_str(&consumer.to_string())?,
                                                    ],
                                                    None,
                                                    1000,
                                                )
                                                .await?;
                                            let signature = keystore
                                                .sign_secure(&signer, &verify_call, Intent::default())?;
                                            let response = sui
                                                .quorum_driver()
                                                .execute_transaction(
                                                    Transaction::from_data(verify_call, Intent::default(), signature).verify()?,
                                                    Some(ExecuteTransactionRequestType::WaitForLocalExecution),
                                                )
                                                .await?;
                                            println!("verify_response: {:?}", response);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    } 
                }
                _ => {}
            }
        }
    }
}

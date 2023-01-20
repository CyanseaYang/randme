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

use fastcrypto::bls12381::min_sig::*;
use fastcrypto::{traits::{KeyPair, Signer, EncodeDecodeBase64}};
//use fastcrypto::encoding::{Encoding, Hex};
use anyhow::anyhow;
use std::{fs, str::FromStr, path::Path};
//use rand::thread_rng;

const PACKAGEID: &str = "0x6e37bcfe8f8e11ba7f6b8a66d5d0b65208a42a4c";
const VERKEYID: &str = "0x5a28d201427d17cbbe3ab8da3dac889d76aafc28";

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

    let path = Path::new("./blskeystore");
    /*let bls_kp = BLS12381KeyPair::generate(&mut thread_rng());
    let contents = bls_kp.encode_base64();
    fs::write(path, contents)?;
   
    let admin_verkey = sui
        .transaction_builder()
        .move_call(
            signer,
            package_id,
            "vrf",
            "admin_verkey",
            vec![],
            vec![
                SuiJsonValue::from_str(VERKEYID)?,
                SuiJsonValue::new(bls_kp.public().as_ref().into())?,    
            ],
            None,
            1000,
        )
        .await?;
    let signature = keystore
        .sign_secure(&signer, &admin_verkey, Intent::default())?;
    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(admin_verkey, Intent::default(), signature).verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;
    println!("admin_verkey_response: {:?}", response);*/

    let contents = fs::read_to_string(path)?;
    let bls_kp = BLS12381KeyPair::decode_base64(contents.as_str().trim()).map_err(|e| anyhow!(e))?;
    //println!("bls12381 public key: 0x{}", &Hex::encode(bls_kp.public().as_ref()));

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
                                            
                                            println!("build verify transaction...");
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
                                                        SuiJsonValue::from_str(&format!("\"{seed}\""))?,
                                                        SuiJsonValue::from_str(&consumer.to_string())?,
                                                        SuiJsonValue::from_str(VERKEYID)?,
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

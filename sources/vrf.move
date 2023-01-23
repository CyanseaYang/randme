// Copyright (c) RandMe
// SPDX-License-Identifier: Apache-2.0

module randme::vrf {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::event;
    use sui::transfer;
    use sui::bls12381;
    use sui::bcs;
    use std::hash::sha3_256;
    use std::vector;
    
    const EInvalidSignature: u64 = 1;

    struct RequestEvent has copy, drop {
        seed: u64,
        consumer: address,
    }

    struct Randomness has key {
        id: UID,
        number: u64,
        seed: u64,
    }

    struct VerificationKey has key {
        id: UID,
        owner: address,
        bls_pubkey: vector<u8>,
    }

    fun init(ctx: &mut TxContext) {
        transfer::share_object(VerificationKey {
            id: object::new(ctx),
            owner: tx_context::sender(ctx),
            bls_pubkey: vector::empty(),
        })
    }

    public entry fun admin_verkey(verkey: &mut VerificationKey, pubkey: vector<u8>, ctx: &mut TxContext) {
        assert!(verkey.owner == tx_context::sender(ctx), 0);
        verkey.bls_pubkey = pubkey;
    }

    public fun request(seed: u64, consumer: address) {
        event::emit(RequestEvent { seed, consumer })
    }

    public fun fulfill(randomness: Randomness): (u64, u64) {
        let Randomness { id, number, seed } = randomness;
        object::delete(id);
        (number, seed)
    }

    public entry fun verify(sig: vector<u8>, seed: u64, consumer: address, verkey: &VerificationKey, ctx: &mut TxContext) {
        assert!(verkey.owner == tx_context::sender(ctx), 0);
        let msg = bcs::to_bytes(&seed);
        vector::append(&mut msg, bcs::to_bytes(&consumer)); 
        assert!(bls12381::bls12381_min_sig_verify(&sig, &verkey.bls_pubkey, &msg), EInvalidSignature);
        
        let output = bcs::new(sha3_256(sig));
        let number = bcs::peel_u64(&mut output);
        let randomness = Randomness {
            id: object::new(ctx),
            number,
            seed,
        };
        transfer::transfer(randomness, consumer);
    }
}

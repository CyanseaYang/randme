module client::client {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use randme::vrf::{Self, Randomness};

    struct Count has key, store {
        id: UID,
        seed: u64,
    }

    fun init(ctx: &mut TxContext) {
        transfer::share_object(
            Count { id: object::new(ctx), seed: 0 }
        )
    }

    public entry fun request_randme(count: &mut Count, ctx: &mut TxContext) {
        count.seed = count.seed + 1;
        vrf::request(count.seed, tx_context::sender(ctx));
    }

    public entry fun fulfill_randme(randomness: Randomness) {
        let (number, seed) = vrf::fulfill(randomness);
        // use random number
        if (number > seed) {}
    }
}
module client::client {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use randme::vrf::{Self, Randomness};

    struct Counter has key {
        id: UID,
        value: u64,
    }

    fun init(ctx: &mut TxContext) {
        transfer::share_object(
            Counter { id: object::new(ctx), value: 0 }
        )
    }

    public entry fun request_randme(counter: &mut Count, ctx: &mut TxContext) {
        counter.value = counter.value + 1;
        vrf::request(counter.value, tx_context::sender(ctx));
    }

    public entry fun fulfill_randme(randomness: Randomness) {
        let (number, seed) = vrf::fulfill(randomness);
        // use random number
    }
}
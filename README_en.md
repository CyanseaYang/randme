# RandMe - Verifiable random number generator on SUI

RandMe VRF generates verifiable random numbers on the SUI chain, which can be used for various on-chain businesses.

The user contract on SUI needs to use the RandMe VRF contract, add it to Move.toml:
	
	randme = { git = "https://github.com/CyanseaYang/randme.git", rev = "master" }
Initiate a random number request in the user contract file:
	
	use randme::vrf::{Self, Randomness};
	
	vrf::request(seed, user_address);
The request function of the VRF contract receives two parameters, the seed of type u64 and the user's SUI address. The seed is defined by the user contract, such as defining a shared object for counting, incrementing the count each time it is used, and using the count as a seed. After vrf::request receives the request, it will emit a RequestEvent event, which includes the seed and user address.

The offline oracle machine is responsible for monitoring the events sent by the VRF contract. Once the RequestEvent is detected, the job will be started, and the seed and user address will be BCS encoded as the original message, and the BLS12381 key pair registered in the VRF contract in advance will be used to process the message. Make a signature and generate a BLS12381 Signature. Then submit the BLS signature, BLS public key, and seed and user addresses to the verify function in the VRF contract.

The verify function of the VRF contract verifies the submitted BLS signature. Once the verification is passed, the BLS signature is hashed using sha2_256 to generate a random number output, and the 256-bit output is converted to 64-bit to obtain a 64-bit random number. Then generate a Randomness SUI object, the object field includes a 64-bit random number and the seed provided by the user contract, and send the Randomness object to the submitted user SUI address.

To process the received Randomness object in real time, the user needs to monitor the NewObject event of the VRF contract. Once it is detected that the recipient of the NewObject is the user himself, it means that he already has the Randomness object. Here is the rust sample code:

	let filters = vec![
      	SuiEventFilter::Module("vrf".to_string()),
      	SuiEventFilter::EventType(EventType::NewObject),
	];
	let mut subscribe = sui
    	.event_api()
    	.subscribe_event(SuiEventFilter::All(filters))
    	.await?;
		......
 	match recipient {
     	Owner::AddressOwner(address) => {
         		if &address == &my_address {
         		...... 
The user submits the received object to a function in the user contract. The user contract needs to provide a function whose parameter is the Randomness object. This function passes the object to the fulfill function of the VRF contract. The fulfill function unpacks the Randomness object and obtains 64 Bit random number and seed, returned to the user contract. User contract function code example:

	public entry fun fulfill_randme(randomness: Randomness) {
    	let (number, seed) = vrf::fulfill(randomness);
    	// use random number
	}

## Flow chart
![](https://raw.githubusercontent.com/CyanseaYang/randme/master/flow.png)

## Follow-up
* The oracle machine adopts BLS threshold signature and MPC(multi-party compute) mode.
* The robustness of the oracle machine can maintain stable operation for a long time.
* Add charging function and subscription function to VRF contract.

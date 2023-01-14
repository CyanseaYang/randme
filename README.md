# RandMe - SUI 上的可验证随机数生成器

RandMe VRF 生成 SUI 链上可验证的随机数，可验证随机数可用于多种链上业务。

SUI 上的用户合约要使用RandMe VRF合约，在Move.toml中加入：
	
	randme = { git = "https://github.com/CyanseaYang/randme.git", rev = "master" }
在用户合约文件中发起随机数请求：
	
	use randme::vrf::{Self, Randomness};
	
	vrf::request(seed, user_address);
VRF合约的request函数接收两个参数，类型为u64的种子和用户的SUI地址。种子由用户合约自行定义，比如定义一个用于计数的共享对象，每次使用时递增计数，将计数作为种子。vrf::request在收到请求后，会发射一个RequestEvent事件，其中包括种子和用户地址。

线下预言机负责监听VRF合约发出的事件，一旦监听到RequestEvent，就启动作业，对种子和用户地址做BCS编码作为原始消息，并使用提前在VRF合约中注册过的BLS12381密钥对，对消息做签名，生成BLS12381 Signature。然后将BLS签名、BLS公钥、以及种子和用户地址提交给VRF合约中的verify函数。

VRF合约的verify函数验证提交的BLS签名。一旦验证通过，就使用sha2_256对BLS签名做哈希运算，生成随机数输出，并将256位的输出转换为64位，得到一个64位随机数。然后生成一个Randomness SUI对象，对象字段包括64位随机数和用户合约提供的种子，将Randomness对象发送给提交的用户SUI地址。

用户要实时处理接收到的Randomness对象，需监听VRF合约的NewObject事件，一旦检测到NewObject的接收方是自己，就表示自己拥有了Randomness对象。下面是rust示例代码：

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
用户将收到的对象提交给用户合约中的一个函数，用户合约需要提供一个参数为Randomness对象的函数，该函数将对象传递给VRF合约的fulfill函数，fulfill函数解包Randomness对象，得到其中的64位随机数和种子，返回给用户合约。用户合约函数代码示例：

	public entry fun fulfill_randme(randomness: Randomness) {
    	let (number, seed) = vrf::fulfill(randomness);
    	// use random number
	}

## 流程图
![](https://raw.githubusercontent.com/CyanseaYang/randme/master/flow.png)

## 后续工作
* 预言机采用BLS阈值签名和MPC多方计算模式。
* 预言机的鲁棒性，能够保持长时间稳定运行。
* 合约中添加收费功能和订阅功能。

//改名字
use crate as pallet_kitties;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64},
};

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},

		//因为我们使用了其它pallet提供的功能来实现kitty的dna，所以这里我们也要实现该pallet
		//但是它也是从support中引入的，为什么就变成了独立的pallet，要实现呢
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		//Balance 模块也是引入的，所以也要实现
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		//实现所定义的pallet
		KittiesModule: pallet_kitties::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// 指明类型
type Balance = u64;
// 实现引入的pallet
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

//这里trait还需要特别去实现一下，可以在runtime中找到范例
impl pallet_randomness_collective_flip::Config for Test {}

//定义常规参数
parameter_types! {
	pub const KittyPrice: u64 = 10;
}

//先实现pallet，runtime中定义的关联类型都必须要
impl pallet_kitties::Config for Test {
	type Event = Event;
	//这里照旧需要指定关联类型的具体类型是什么
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type KittyIndex = u32;
	type MaxKittyIndex = ConstU32<3>;
	type KittyPrice = KittyPrice;
}

// Build genesis storage according to the mock runtime.
// 如下内容也需要重新构建
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ts = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(0, 100), (1, 25), (2, 1)] }
		.assimilate_storage(&mut ts)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(ts);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

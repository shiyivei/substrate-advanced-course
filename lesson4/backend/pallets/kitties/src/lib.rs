#![cfg_attr(not(feature = "std"), no_std)] //编译标签

//pallet 导出
pub use pallet::*;

use sp_core::crypto::KeyTypeId;

//测试环境配置模块
#[cfg(test)]
mod mock;

//测试模块
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

//构建加密账户和依赖
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"kty!");

pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct KittiesAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for KittiesAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for KittiesAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet] //开发pellet所用的宏
pub mod pallet {

	//引入类型或函数
	use frame_support::traits::{Randomness, ReservableCurrency};
	use frame_support::{log, pallet_prelude::*, traits::Currency}; //Currency,固定引入方法
	use frame_system::pallet_prelude::*; //比如一些方便签名和验证的方法
	use sp_io::hashing::blake2_128;

	use frame_support::inherent::Vec;
	use frame_system::offchain::SendSignedTransaction;
	use frame_system::offchain::{AppCrypto, CreateSignedTransaction, Signer};
	use sp_io::offchain_index;
	use sp_runtime::offchain::storage::StorageValueRef;
	use sp_runtime::traits::Zero;

	//定义新类型，并想为其实现一些必要的trait时，可以直接引用现成的类型，无需重新定义trait
	use sp_runtime::traits::{AtLeast32Bit, Bounded, CheckedAdd}; //引入trait

	//定义一个类型别名,使用Currency这个trait，先得引入，如下也是常见写法，凡是涉及到金钱的，这个类型必不可少
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	//定义类型(类型别名，在业务中易于识别),本来这个类型我们是定义在trait外部的，但是现在改为定义在trait内部，然后用的时候指定
	// type KittyIndex = u32;

	//定义函数
	//函数要使用trait中的关联类型，必须先为定义trait为泛型约束
	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		//取0
		0_u8.into()
	}

	//定义业务数据
	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty {
		pub dna: [u8; 16],
		pub asset: u32,
	}

	//定义接口
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		//定义关联类型 Currency
		type Currency: ReservableCurrency<Self::AccountId>;

		//在config中定义类型，并为其增加必须的trait，然后在runtime中指定
		//例子：无符号整数类型定义模版
		type KittyIndex: Parameter + AtLeast32Bit + Default + Copy + Bounded + MaxEncodedLen;

		//定义常量
		#[pallet::constant]
		type MaxKittyIndex: Get<u32>;

		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;

		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	//定义Pallet结构体
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)] //生成包含所有存储项的接口
	pub struct Pallet<T>(_);

	//定义存储
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	//同样的，不管是在函数中使用trait中的关联类型还是在新的类型中使用，都需要将trait作为泛型约束
	pub type NextKittyId<T: Config> =
		StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue<T>>;

	//定义存储
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	//定义存储
	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	//定义存储
	#[pallet::storage]
	# [pallet::getter(fn all_kitties)]
	pub type AllKitties<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::KittyIndex, T::MaxKittyIndex>,
		ValueQuery,
	>;

	//定义执行成功事件
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		KittyBred(T::AccountId, T::KittyIndex, Kitty),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
	}

	//定义执行失败错误
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		KittyIdOverflow,
		NotOwner,
		SameKittyId,
		NotEnoughBalance,
		OwnTooManyKitties,
	}

	const ONCHAIN_TX_KEY: &[u8] = b"kitty_pallet::indexing01";

	#[derive(Debug, Encode, Decode, Default)]
	// struct IndexingData<T: Config>(T::KittyIndex);
	struct IndexingData<T: Config>(T::KittyIndex);

	#[pallet::hooks]
	// 沉睡8000ms，根据block number 奇偶性确定kitty id
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hello World from offchain workers!: {:?}", block_number);

			// 获取key
			let key = Self::derived_key(block_number);

			// log::info!("the new store key is {:?}", key);

			// 读取存储内容
			let storage_content = StorageValueRef::persistent(&key);

			//读取内容并更新
			if let Ok(Some(data)) = storage_content.get::<IndexingData<T>>() {
				// if let Some((Some(data)) = storage_ref.get::<IndexingData>() {
				// Sleep 8000ms to simulate heavy calculation for kitty asset index.
				log::info!("It needs a lot of time to create a kitty {:?}", block_number);
				let timeout = sp_io::offchain::timestamp()
					.add(sp_runtime::offchain::Duration::from_millis(8000));
				sp_io::offchain::sleep_until(timeout);

				log::info!("new kitty has been created and stored both onchain and offchain");

				let kitty_id = data.0.into();

				log::info!("get kitty index from offchain {:?}", kitty_id);

				if block_number % 2u32.into() != Zero::zero() {
					_ = Self::send_signed_tx(kitty_id, 100);
					log::info!("now the block number is odd, let's update kitty's asset as 100, and kitty id is {:?}",kitty_id);
				} else {
					_ = Self::send_signed_tx(kitty_id, 200);
					log::info!("now the block number is even, let's update kitty's asset as 200, and kitty id is {:?}",kitty_id);
				}
			} else {
				log::info!("No off-chain indexing data retrieved.");
			}

			log::info!("Leave from offchain workers!: {:?}", block_number);
		}
	}

	//定义执行逻辑
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		//创建kitty
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			//验证签名
			let who = ensure_signed(origin)?;

			//获取kitty价格
			let kitty_price = T::KittyPrice::get();

			//判断创建者的钱包余额是否足够，这里是锁仓create
			ensure!(T::Currency::can_reserve(&who, kitty_price), Error::<T>::NotEnoughBalance);

			//获取 kitty_id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			//获取dna
			let dna = Self::random_value(&who);
			//创造新的 kitty
			let kitty = Kitty { dna, asset: 0 };

			//质押
			T::Currency::reserve(&who, kitty_price)?;

			//存储kitty和id
			Kitties::<T>::insert(kitty_id, &kitty);

			//存储kittyId和所有者
			KittyOwner::<T>::insert(kitty_id, &who);

			//获取下一个kitty的id
			let next_kitty_id = kitty_id
				.checked_add(&(T::KittyIndex::from(1_u8)))
				.ok_or(Error::<T>::KittyIdOverflow)
				.unwrap();

			//设置下一个kitty_id
			NextKittyId::<T>::set(next_kitty_id);

			AllKitties::<T>::try_mutate(&who, |ref mut kitties| {
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(), DispatchError>(())
			})?;

			// 同时把数据存到链下存储中
			Self::store_kitty_to_indexing(kitty_id);

			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));

			Ok(())
		}

		//繁殖kitty
		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//获取kitty价格
			let kitty_price = T::KittyPrice::get();

			//判断钱包余额是否足够
			ensure!(T::Currency::can_reserve(&who, kitty_price), Error::<T>::NotEnoughBalance);

			// 确保不是相同的kitty
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			//通过kitty_id先获取两只kitty
			let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

			//获取新的kitty的id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// 得到随机的dna
			let selector = Self::random_value(&who);

			// 定义数据
			let mut data = [0u8; 16];

			for i in 0..kitty_1.dna.len() {
				data[i] = (kitty_1.dna[i] & selector[i]) | (kitty_2.dna[i] & selector[i]);
			}

			// 生成新kitty
			let new_kitty = Kitty { dna: data, asset: 0 };

			//质押
			T::Currency::reserve(&who, kitty_price)?;

			//保存id
			Kitties::<T>::insert(kitty_id, &new_kitty);
			//保存所有者
			KittyOwner::<T>::insert(kitty_id, &who);

			//获取新的id
			let next_kitty_id = kitty_id
				.checked_add(&(T::KittyIndex::from(1_u32)))
				.ok_or(Error::<T>::KittyIdOverflow)
				.unwrap();

			NextKittyId::<T>::set(next_kitty_id);

			AllKitties::<T>::try_mutate(&who, |ref mut kitties| {
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(), DispatchError>(())
			})?;

			Self::store_kitty_to_indexing(kitty_id);

			Self::deposit_event(Event::KittyBred(who, kitty_id, new_kitty));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//获取kitty价格
			let kitty_price = T::KittyPrice::get();

			//判断钱包余额是否足够
			ensure!(T::Currency::can_reserve(&who, kitty_price), Error::<T>::NotEnoughBalance);

			//获取要转移的kitty，验证kitty_id有效性
			Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			//确保是拥有者
			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

			//释放和锁定金钱
			T::Currency::unreserve(&who, kitty_price);
			T::Currency::reserve(&new_owner, kitty_price)?;

			//更改owner，键相同，值覆盖
			KittyOwner::<T>::insert(kitty_id, &new_owner);

			//移除某个account下的kitties信息
			AllKitties::<T>::try_mutate(&who, |ref mut kitties| {
				let index = kitties.iter().position(|&r| r == kitty_id).unwrap();
				kitties.remove(index);
				Ok::<(), DispatchError>(())
			})?;

			//更改新的owner
			AllKitties::<T>::try_mutate(&new_owner, |ref mut kitties| {
				// print!("### before push kitty to Kitties");
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				// print!("### after push kitty to Kitties");
				Ok::<(), DispatchError>(())
			})?;

			Self::deposit_event(Event::KittyTransferred(who, new_owner, kitty_id));

			Ok(())
		}
		#[pallet::weight(0)]
		//更新链上数据
		pub fn update_kitty(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			asset: u32,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			let kitty = Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			let new_kitty = Kitty { dna: kitty.dna, asset };

			Kitties::<T>::insert(kitty_id, &new_kitty);

			Ok(().into())
		}
	}

	//定义辅助性的函数,这些函数不需要weights，但是实际上他们会被需要weights的函数在其内部所调用，间接的也说明并不是无成本的
	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn get_next_id() -> Result<T::KittyIndex, ()> {
			let kitty_id = Self::next_kitty_id();
			match kitty_id {
				_ if T::KittyIndex::max_value() <= kitty_id => Err(()),
				val => Ok(val),
			}
		}

		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

		// 辅助函数
		fn derived_key(block_number: T::BlockNumber) -> Vec<u8> {
			block_number.using_encoded(|encoded_bn| {
				ONCHAIN_TX_KEY
					.clone()
					.into_iter()
					.chain(b"/".into_iter())
					.chain(encoded_bn)
					.copied()
					.collect::<Vec<u8>>()
			})
		}

		// 写入存储
		fn store_kitty_to_indexing(kitty_id: T::KittyIndex) {
			let key = Self::derived_key(frame_system::Pallet::<T>::block_number());

			// log::info!("the store key is {:?}", key);

			let data: IndexingData<T> = IndexingData(kitty_id);
			offchain_index::set(&key, &data.encode());
			log::info!("kitty id has been stored in offchain storage:{:?}", kitty_id);
		}

		fn send_signed_tx(kitty_id: T::KittyIndex, payload: u32) -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if !signer.can_sign() {
				return Err(
					"No local accounts available. Consider adding one via `author_insertKey` RPC.",
				);
			}

			log::info!("updating kitty asset, {:?}", kitty_id);
			// update_kitty info
			let results = signer.send_signed_transaction(|_account| Call::update_kitty {
				kitty_id,
				asset: payload,
			});

			for (acc, res) in &results {
				match res {
					Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, (kitty_id, payload)),
					Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
				}
			}

			Ok(())
		}
	}
}

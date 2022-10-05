#![cfg_attr(not(feature = "std"), no_std)] //编译标签

//pallet 导出
pub use pallet::*;

//测试环境配置模块
// #[cfg(test)]
// mod mock;

//测试模块
// #[cfg(test)]
// mod tests;

#[frame_support::pallet] //开发pellet所用的宏
pub mod pallet {

	//引入类型或函数
	use frame_support::traits::{Randomness, ReservableCurrency};
	use frame_support::{pallet_prelude::*, traits::Currency}; //Currency,固定引入方法
	use frame_system::pallet_prelude::*; //比如一些方便签名和验证的方法
	use sp_io::hashing::blake2_128;

	//定义新类型，并想为其实现一些必要的trait时，可以直接引用现成的类型，无需重新定义trait
	use sp_runtime::traits::{AtLeast32Bit, Bounded,CheckedAdd}; //引入trait

	//定义一个类型别名,使用Currency这个trait，先得引入，如下也是常见写法，凡是涉及到金钱的，这个类型必不可少
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	//定义接口
	#[pallet::config]
	pub trait Config: frame_system::Config {
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
	}

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
	pub struct Kitty(pub [u8; 16]);

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

	//定义hook
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
			ensure!(T::Currency::can_reserve(&who,kitty_price),Error::<T>::NotEnoughBalance);

			//获取 kitty_id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			//获取dna
			let dna = Self::random_value(&who);
			//创造新的 kitty
			let kitty = Kitty(dna);

			//质押
			T::Currency::reserve(&who,kitty_price)?;

			//存储kitty和id
			Kitties::<T>::insert(kitty_id, &kitty);

			//存储kittyId和所有者
			KittyOwner::<T>::insert(kitty_id, &who);

			//获取下一个kitty的id
			let next_kitty_id = kitty_id.checked_add(&(T::KittyIndex::from(1_u32))).ok_or(Error::<T>::KittyIdOverflow).unwrap();
			
			//设置下一个kitty_id
			NextKittyId::<T>::set(next_kitty_id);

			AllKitties::<T>::try_mutate(&who,|ref mut kitties|{
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(),DispatchError>(())
			})?;


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
			ensure!(T::Currency::can_reserve(&who,kitty_price),Error::<T>::NotEnoughBalance);

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

			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & selector[i]);
			}

			// 生成新kitty
			let new_kitty = Kitty(data);

			//质押
			T::Currency::reserve(&who,kitty_price)?;

			//保存id
			Kitties::<T>::insert(kitty_id, &new_kitty);
			//保存所有者
			KittyOwner::<T>::insert(kitty_id, &who);

			//获取新的id
			let next_kitty_id = kitty_id.checked_add(&(T::KittyIndex::from(1_u32))).ok_or(Error::<T>::KittyIdOverflow).unwrap();
			
			NextKittyId::<T>::set(next_kitty_id);

			AllKitties::<T>::try_mutate(&who,|ref mut kitties|{
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(),DispatchError>(())
			})?;

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
			ensure!(T::Currency::can_reserve(&who,kitty_price),Error::<T>::NotEnoughBalance);

			//获取要转移的kitty，验证kitty_id有效性
			Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			//确保是拥有者
			ensure!(Self::kitty_owner(kitty_id)== Some(who.clone()), Error::<T>::NotOwner);

			//释放和锁定金钱
			T::Currency::unreserve(&who,kitty_price);
			T::Currency::reserve(&new_owner,kitty_price)?;

			//更改owner，键相同，值覆盖
			KittyOwner::<T>::insert(kitty_id, &new_owner);

			//移除某个account下的kitties信息
			AllKitties::<T>::try_mutate(&who,|ref mut kitties|{
				let index = kitties.iter().position(|&r| r == kitty_id).unwrap();
				kitties.remove(index);
				Ok::<(),DispatchError>(())
			})?;

			//更改新的owner
			AllKitties::<T>::try_mutate(&new_owner,|ref mut kitties|{
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(),DispatchError>(())
			})?;

			Self::deposit_event(Event::KittyTransferred(who, new_owner, kitty_id));

			Ok(())
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
	}
}

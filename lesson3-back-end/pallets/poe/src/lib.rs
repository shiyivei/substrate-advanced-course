#![cfg_attr(not(feature = "std"), no_std)] //编译标签

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet] //开发pellet所用的宏
pub mod pallet {

	//从依赖中使用下面的路径引入方法或者类型
	use frame_support::pallet_prelude::*; //比如get接口
	use frame_system::pallet_prelude::*; //比如一些方便签名和验证的方法
	use sp_std::prelude::*; //引入要使用的vector

	//定义pallet的配置接口(trait),要求继承系统配置,这样就能够继承到一些类型，如 block number，hash，account id
	#[pallet::config]
	pub trait Config: frame_system::Config {
		//添加关联类型
		#[pallet::constant]
		type MaxClaimLength: Get<u32>; //使用接口获得的
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	//定义模块所需要的结构体
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)] //生成包含所有存储项的接口
	pub struct Pallet<T>(_);

	//定义存储
	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	//定义事件
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimChanged(T::AccountId, Vec<u8>),
	}

	//定义错误
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	//定义hook
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//新建存储
		#[pallet::weight(0)]
		//这里claim的类型是Vec<8>,它通常是一个代表具体内容的哈希值，因为链上存储十分宝贵，所以一般村放哈希值
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			//判断是否超出了字长
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			//判断是否已经存储
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

			//执行存储
			Proofs::<T>::insert(
				bounded_claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number()),
			);

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		//撤销存证
		#[pallet::weight(0)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Proofs::<T>::remove(&bounded_claim);

			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		//转移存证
		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			let (owner, _block_number) =
				Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Proofs::<T>::insert(&bounded_claim, (dest, frame_system::Pallet::<T>::block_number()));
			//Proofs::<T>::mutate(&bounded_claim, |v| *v = Some((dest, _block_number)));

			Self::deposit_event(Event::ClaimChanged(sender, claim));

			Ok(().into())
		}
	}
}

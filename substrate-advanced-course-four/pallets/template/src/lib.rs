#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

//发送未签名交易
use frame_system::offchain::SubmitTransaction;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};

//发送http请求
// use sp_runtime::offchain::{http, Duration};

// use serde::{Deserialize, Deserializer};

//ocw signed tx dependencies
// use frame_system::offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer};
// use sp_core::crypto::KeyTypeId;

// pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");

// pub mod crypto {
// 	use super::KEY_TYPE;

// 	use sp_core::sr25519::Signature as Sr25519Signature;
// 	use sp_runtime::{
// 		app_crypto::{app_crypto, sr25519},
// 		traits::Verify,
// 		MultiSignature, MultiSigner,
// 	};
// 	app_crypto!(sr25519, KEY_TYPE);
// 	pub struct OcwAuthId;

// 	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
// 		type RuntimeAppPublic = Public;
// 		type GenericSignature = sp_core::sr25519::Signature;
// 		type GenericPublic = sp_core::sr25519::Public;
// 	}

// 	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
// 		for OcwAuthId
// 	{
// 		type RuntimeAppPublic = Public;
// 		type GenericSignature = sp_core::sr25519::Signature;
// 		type GenericPublic = sp_core::sr25519::Public;
// 	}
// }

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	// use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_none, pallet_prelude::*};
	// use sp_std::vec;

	// #[derive(Deserialize, Encode, Decode)]
	// struct GithubInfo {
	// 	#[serde(deserialize_with = "de_string_to_bytes")]
	// 	login: Vec<u8>,
	// 	#[serde(deserialize_with = "de_string_to_bytes")]
	// 	blog: Vec<u8>,
	// 	public_repos: u32,
	// }

	// pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
	// where
	// 	D: Deserializer<'de>,
	// {
	// 	let s: &str = Deserialize::deserialize(de)?;
	// 	Ok(s.as_bytes().to_vec())
	// }

	// use core::{convert::TryInto, fmt};
	// impl fmt::Debug for GithubInfo {
	// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	// 		write!(
	// 			f,
	// 			"{{login:{},blog:{},public_repos:{}}}",
	// 			sp_std::str::from_utf8(&self.login).map_err(|_| fmt::Error)?,
	// 			sp_std::str::from_utf8(&self.blog).map_err(|_| fmt::Error)?,
	// 			&self.public_repos
	// 		)
	// 	}
	// }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + frame_system::offchain::SendTransactionTypes<Call<Self>>
	{
		// pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		//签名交易
		// #[pallet::weight(0)]
		// pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {
		// 	let _who = ensure_signed(origin)?;

		// 	log::info!("in submit_data call: {:?}", payload);

		// 	Ok(().into())
		// }

		//未签名交易
		#[pallet::weight(0)]
		pub fn submit_data_unsigned(origin: OriginFor<T>, n: u64) -> DispatchResult {
			ensure_none(origin)?;

			log::info!("in submit_data_unsigned: {:?}", n);

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hello World from offChain workers!:{:?}", block_number);
			// 未交位签名交易

			let value: u64 = 42;
			let call = Call::submit_data_unsigned { n: value };

			_ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(
				|_| {
					log::error!("Failed in offchain_unsigned_tx");
				},
			);

			log::info!("Leave from offChain workers!:{:?}", block_number);

			//提交签名交易

			// let payload: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
			// _ = Self::send_signed_tx(payload);

			// log::info!("Leave from offChain workers!:{:?}", block_number)

			// 发送httlp 请求
			// if let Ok(info) = Self::fetch_github_info() {
			// 	log::info!("Github Info: {:?}", info);
			// } else {
			// 	log::info!("Error while fetch github info!");
			// }

			// log::info!("leave from offChain workers!:{:?}", block_number);

			//验证offchain worker 跨块执行
			// let timeout =
			// 	sp_io::offchain::timestamp().add(sp_runtime::offchain::Duration::from_millis(8000));

			// sp_io::offchain::sleep_until(timeout);

			// log::info!("Leave from offChain workers!: {:?}", block_number);

			//在奇数块写数据，在偶数块度数据
			// if block_number % 2u32.into() != Zero::zero() {
			// 	// odd
			// 	let key = Self::derive_key(block_number);
			// 	// 声明存储
			// 	let val_ref = StorageValueRef::persistent(&key);

			// 	//get a local value random values
			// 	let random_slice = sp_io::offchain::random_seed();

			// 	//get a local timestamp
			// 	let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();

			// 	// combine to a tuple and print
			// 	let value = (random_slice, timestamp_u64);
			// 	log::info!("in odd block, value to write:{:?}", value);

			// 	struct StateError;

			// 	let res = val_ref.mutate(
			// 		|val: Result<Option<([u8; 32], u64)>, StorageRetrievalError>| ->Result < _,
			// 		StateError > {
			// 			match val {
			// 				Ok(Some(_)) => Ok(value),
			// 				_ => Ok(value),
			// 			}
			// 		});
			// 	match res {
			// 		Ok(value) => {
			// 			log::info!("in odd block,mutate successfully:{:?}", value);
			// 		},
			// 		Err(MutateStorageError::ValueFunctionFailed(_)) => (),
			// 		Err(MutateStorageError::ConcurrentModification(_)) => (),
			// 	}

			// //write or mutate tuple content to key
			// // val_ref.set(&value);
			// } else {
			// 	//even
			// 	let key = Self::derive_key(block_number - 1u32.into());
			// 	let mut val_ref = StorageValueRef::persistent(&key);

			// 	//get form db by key
			// 	if let Ok(Some(value)) = val_ref.get::<([u8; 32], u64)>() {
			// 		//print value
			// 		log::info!("in even block, value read: {:?}", value);
			// 		//delete that key

			// 		val_ref.clear();
			// 	}

			// 	log::info!("Leave from offchain workers!: {:?}", block_number)
			// }
		}

		fn on_initialize(_n: T::BlockNumber) -> Weight {
			log::info!("in on_initialize!");
			let weight = Weight::from_ref_time(0);
			weight
		}

		fn on_finalize(_n: T::BlockNumber) {
			log::info!("in on_finalize!");
		}

		fn on_idle(_n: T::BlockNumber, _remain_weight: Weight) -> Weight {
			log::info!("in on_idle!");
			let weight = Weight::from_ref_time(0);
			weight
		}
	}
	//未签名交易
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			if let Call::submit_data_unsigned { n: _ } = call {
				ValidTransaction::with_tag_prefix("ExampleOffchainWorker")
					.priority(10000)
					.and_provides(1)
					.longevity(3)
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}
	// 本地存储辅助函数
	impl<T: Config> Pallet<T> {
		// 	#[deny(clippy::clone_double_ref)]
		// 	fn derive_key(block_number: T::BlockNumber) -> Vec<u8> {
		// 		block_number.using_encoded(|encoded_bn| {
		// 			b"node-template::storage".iter().chain(encoded_bn).copied().collect::<Vec<u8>>()
		// 		})
		// 	}

		//发送http请求的辅助函数

		// fn fetch_github_info() -> Result<GithubInfo, http::Error> {
		// 	let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
		// 	let request = http::Request::get("https://api.github.com/orgs/substrate-developer-hub");
		// 	let pending = request
		// 		.add_header("User-Agent", "Substrate-Offchain-Worker")
		// 		.deadline(deadline)
		// 		.send()
		// 		.map_err(|_| http::Error::IoError)?;

		// 	let response =
		// 		pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;

		// 	if response.code != 200 {
		// 		log::warn!("Unexpected status code: {}", response.code);
		// 		return Err(http::Error::Unknown);
		// 	}

		// 	let body = response.body().collect::<Vec<u8>>();
		// 	let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
		// 		log::warn!("Not UTF8 body");
		// 		http::Error::Unknown
		// 	})?;

		// 	//parse the response str

		// 	let gh_info: GithubInfo =
		// 		serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

		// 	Ok(gh_info)
		// }

		//发送签名交易的辅助函数

		// fn send_signed_tx(payload: Vec<u8>) -> Result<(), &'static str> {
		// 	let signer = Signer::<T, T::AuthorityId>::all_accounts();
		// 	if !signer.can_sign() {
		// 		return Err(
		// 			"No local accounts available,consider adding one via 'author_insertKey' RPC.",
		// 		);
		// 	}

		// 	let results = signer
		// 		.send_signed_transaction(|_account| Call::submit_data { payload: payload.clone() });

		// 	for (acc, res) in &results {
		// 		match res {
		// 			Ok(()) => log::info!("[{:?}],Submitted data:{:?}", acc.id, payload),
		// 			Err(e) => log::error!("[{:?}],Failed to submit transaction: {:?}", acc.id, e),
		// 		}
		// 	}

		// 	Ok(())
		// }
	}
}

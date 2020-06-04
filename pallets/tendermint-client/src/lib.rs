#![cfg_attr(not(feature = "std"), no_std)]

/// Wormhole TendermintClient Pallet. Allows verification of Tendermint block headers on the substrate chain.

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure};
use frame_system::{self as system, ensure_signed};
use tendermint::{
	block::{
		signed_header::SignedHeader,
	},
	time::Time,
};
use sha2::{Sha256, Digest};
#[macro_use]
extern crate alloc;
use  sp_std::vec::Vec;
use log::{error, debug};

mod types;

use crate::types::{TendermintClient, ConsensusState, TMCreateClientPayload, TMUpdateClientPayload, TMClientStorageWrapper};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TendermintClientModule {

		TMClientStorage: map hasher(blake2_128_concat) Vec<u8> => TMClientStorageWrapper;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Just a dummy event.
		/// Event `ClientCreated`/`ClientUpdated` is declared with a parameter of the type `string` (name), `string` (chainid), `u64` (height)
		/// To emit this event, we call the deposit function, from our runtime functions
		ClientCreated(AccountId, Vec<u8>, Vec<u8>, u64),
		ClientUpdated(AccountId, Vec<u8>, Vec<u8>, u64),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
		/// Item not found in storage.
		ItemNotFound,
		/// Unable to deserialize extrinsic.
		DeserializeError,
		/// Parsing Error occurred
		ParseError,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		/// Client initialisation entry point.
		/// takes tendermint/ibc/tendermint/CreateClient message.
		#[weight = 100_000]
		pub fn init_client(origin, payload: Vec<u8>) -> dispatch::DispatchResult {
			debug!("init client");
			debug!("Submitted payload: {:?}", &payload[..]);
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;
			let r: Result<_, _> = serde_json::from_slice(&payload[..]);
			let container: TMCreateClientPayload = r.map_err(|e| {
			  error!("Deserialization Error: {}", e);
			  Error::<T>::DeserializeError
			})?;
			let header: SignedHeader = container.header.signed_header;
			let trust_period: u64 = container.trusting_period;
			let max_clock_drift: u64 = container.max_clock_drift;
			let unbonding_period: u64 = container.unbonding_period;
			// TODO:  validate header
			// TODO:  validate client name
			// TODO:  validate trust_period
			let state: ConsensusState = ConsensusState{
				signed_header: header.clone(),
				height: header.header.height.value(),
				last_update: Time::now(),
				next_validator_set: container.validator_set
			};

			let tmclient: TendermintClient = TendermintClient{
				state: Some(state.clone()),
				trusting_period: trust_period,
				client_id: container.client_id.clone(),
				max_clock_drift: max_clock_drift,
				unbonding_period: unbonding_period,
				chain_id: header.header.chain_id.as_bytes().to_vec(),
			};

			let mut hasher = Sha256::new();
			hasher.input(&container.client_id);
			let key = hasher.result();

			TMClientStorage::insert(key.as_slice(), TMClientStorageWrapper{client: tmclient.clone()});
			// TODO: does this error if the key already exists?

			// Here we are raising the ClientCreated event
			Self::deposit_event(RawEvent::ClientCreated(who, tmclient.client_id, tmclient.chain_id, state.height));
			Ok(())
		}

		/// Update client entry point.
		#[weight = 100_000]
		pub fn update_client(origin, payload: Vec<u8>) -> dispatch::DispatchResult {

			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let _who = ensure_signed(origin)?;

			let r: Result<_, _> = serde_json::from_slice(&payload[..]);
			let container: TMUpdateClientPayload = r.map_err(|e| {
			  error!("Deserialization Error: {}", e);
			  Error::<T>::DeserializeError
			})?;

			let mut hasher = Sha256::new();
			hasher.input(&container.client_id);
			let key = hasher.result();

			ensure!(TMClientStorage::contains_key(key.as_slice()), Error::<T>::ItemNotFound);

            let mut wrapped_client: TMClientStorageWrapper = TMClientStorage::get(key.as_slice());

			let header: SignedHeader = container.header.signed_header;

			// TODO:  validate header
			let state: ConsensusState = ConsensusState{
				signed_header: header.clone(),
				height: header.header.height.value(),
				last_update: Time::now(),
				next_validator_set: wrapped_client.client.state.unwrap().next_validator_set // TODO: handle validator set changes?
			};
			wrapped_client.client.state = Some(state.clone());
			TMClientStorage::insert(key.as_slice(), wrapped_client.clone());
			Ok(())
		}
	}
}

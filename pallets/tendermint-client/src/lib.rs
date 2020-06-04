#![cfg_attr(not(feature = "std"), no_std)]

/// Wormhole TendermintClient Pallet. Allows verification of Tendermint block headers on the substrate chain.

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure};
use frame_system::{self as system, ensure_signed};
use tendermint::{
	block::{
		signed_header::SignedHeader,
	},
	time::Time,
	//lite::TrustThresholdFraction,
	validator::Set,
};
use sha2::{Sha256, Digest};
#[macro_use]
extern crate alloc;
use  sp_std::vec::Vec;

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
		/// function that can be called by the external world as an extrinsics call
		/// takes Tendermint::SignedHeader, trust_period as Duration and a client_name as String, validates the SignedHeader, creates a TendermintClient and stores it, emitting an event.
		#[weight = 100_000]
		pub fn init_client(origin, payload: Vec<u8>) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;

			let container: TMCreateClientPayload = serde_json::from_slice(&payload[..]).map_err(|_e| Error::<T>::DeserializeError)?;

			let header: SignedHeader = container.header;
			let trust_period: u64 = container.trust_period;
			// validate header
			// validate client name
			// validate trust_period
			let state: ConsensusState = ConsensusState{
				signed_header: header.clone(),
				height: header.header.height.value(),
				last_update: Time::now(),
				next_validator_set: Set::new(vec![]) // TODO: populate this!
			};

			let tmclient: TendermintClient = TendermintClient{
				state: Some(state.clone()),
				trusting_period: trust_period,
				client_name: container.client_name.clone(),
				chain_id: header.header.chain_id.as_bytes().to_vec(),
			};

			let mut hasher = Sha256::new();
			hasher.input(&container.client_name);
			let key = hasher.result();
			TMClientStorage::insert(key.as_slice(), TMClientStorageWrapper{client: tmclient.clone()});

			// Here we are raising the ClientCreated event
			Self::deposit_event(RawEvent::ClientCreated(who, tmclient.client_name, tmclient.chain_id, state.height));
			Ok(())
		}

		/// Another dummy entry point.
		/// takes no parameters, attempts to increment storage value, and possibly throws an error
		#[weight = 100_000]
		pub fn update_client(origin, payload: Vec<u8>) -> dispatch::DispatchResult {

			let container: TMUpdateClientPayload = serde_json::from_slice(&payload[..]).map_err(|_e| Error::<T>::DeserializeError)?;

			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let _who = ensure_signed(origin)?;
			let mut hasher = Sha256::new();
			hasher.input(&container.client_name);
			let key = hasher.result();
			ensure!(TMClientStorage::contains_key(key.as_slice()), Error::<T>::ItemNotFound);

            // Get owner of the claim
            let _wrapped_client: TMClientStorageWrapper = TMClientStorage::get(key.as_slice());

			Ok(())
		}
	}
}

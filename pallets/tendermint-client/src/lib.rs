#![cfg_attr(not(feature = "std"), no_std)]

/// Wormhole TendermintClient Pallet. Allows verification of Tendermint block headers on the substrate chain.

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};
use tendermint::{
	block::{
		signed_header::SignedHeader,
	},
	time::Time,
	//lite::TrustThresholdFraction,
	//validator::Set as TMValidatorSet,
};
use std::time::Duration;
use parse_duration::parse;

mod types;

use crate::types::{TendermintClient, ConsensusState, TMCreateClientPayload, TMUpdateClientPayload};

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
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		TMClientStorage get(fn name): Option<TendermintClient>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Just a dummy event.
		/// Event `ClientCreated`/`ClientUpdated` is declared with a parameter of the type `string` (name), `string` (chainid), `u32` (height)
		/// To emit this event, we call the deposit function, from our runtime functions
		ClientCreated(AccountId, String, String, u32),
		ClientUpdated(AccountId, String, String, u32),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
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
		pub fn init_client(origin, payload: &[u8]) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;

			let container: TMCreateClientPayload = serde_json::from_slice(payload);

			let header: SignedHeader = container.header;
			let trust_period: Duration = parse(container.trust_period.clone());
			// validate header
			// validate client name
			// validate trust_period
			let state: ConsensusState = ConsensusState{
				header: header,
				height: header.header.height,
			};

			let tmclient: TendermintClient = TendermintClient{
				state: state,
				trust_period: trust_period,
				client_name: container.client_name.clone(),
				chain_id: header.header.chain_id,
			};

			TMClientStorage::put(tmclient);

			// Here we are raising the ClientCreated event
			Self::deposit_event(RawEvent::ClientCreated(who, tmclient.client_name, tmclient.chain_id, tmclient.height));
			Ok(())
		}

		/// Another dummy entry point.
		/// takes no parameters, attempts to increment storage value, and possibly throws an error
		#[weight = 100_000]
		pub fn update_client(origin, payload: TMUpdateClientPayload) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let _who = ensure_signed(origin)?;

			match TMClientStorage::get() {
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {

					Ok(())
				},
			}
		}
	}
}

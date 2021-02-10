#![cfg_attr(not(feature = "std"), no_std)]

/// Wormhole TendermintClient Pallet. Allows verification of Tendermint block headers on the substrate chain.
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, ensure};
use frame_system::{self as system, ensure_signed};

use chrono::Utc;
use core::time::Duration;
use tendermint_light_client::{
    validate_initial_signed_header_and_valset, verify_single, LightSignedHeader, LightValidator,
    LightValidatorSet, TrustThresholdFraction, TrustedState,
};

extern crate alloc;
extern crate core;
extern crate std;
use log::{debug, error};
use sp_std::vec::Vec;

mod types;

use crate::types::{
    ConsensusState, TMClientInfo, TMClientStorageWrapper, TMCreateClientPayload,
    TMUpdateClientPayload, TendermintClient,
};

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

decl_storage! {
    trait Store for Module<T: Trait> as TendermintClientModule {
        /// Stores state for each client created by its client_id
        TMClientStorage: map hasher(blake2_128_concat) Vec<u8> => TMClientStorageWrapper;
        /// Stores information about each client's state by its client_id
        ClientInfoMap get(fn client_info): map hasher(blake2_128_concat) Vec<u8> => TMClientInfo;
        /// Lists all available clients
        AvailableClients get(fn clients): Vec<Vec<u8>>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Height = u64,
    {
        /// Event `ClientCreated`/`ClientUpdated` is declared with a parameter of the type `string` (name), `string` (chainid), `u64` (height)
        /// and is fired when a client is created/updated respectively.
        ClientCreated(AccountId, Vec<u8>, Vec<u8>, Height),
        ClientUpdated(AccountId, Vec<u8>, Vec<u8>, Height),
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
        /// Client already initialized,
        ClientAlreadyInitialized,
        /// Unable to deserialize extrinsic.
        DeserializeError,
        /// Parsing Error occurred
        ParseError,
        /// Error occurred validating block.
        ValidationError,
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
        /// takes json encoded `TMCreateClientPayload` struct.
        #[weight = 100_000]
        pub fn init_client(origin, payload: Vec<u8>) -> dispatch::DispatchResult {
            // Check it was signed
            let signer = ensure_signed(origin)?;

            debug!("Submitted client initialization payload: {:?}", &payload[..]);

            let init_client_payload: TMCreateClientPayload = serde_json::from_slice(&payload[..]).map_err(|e| {
                error!("Deserialization Error: {}", e);
                Error::<T>::DeserializeError
            })?;

            // Validating if client already exists
            ensure!(!TMClientStorage::contains_key(init_client_payload.client_id.as_bytes().to_vec()), Error::<T>::ClientAlreadyInitialized);

            let header: LightSignedHeader = init_client_payload.header.signed_header;
            let validator_set: LightValidatorSet<LightValidator> = init_client_payload.header.validator_set;
            let chain_id = header.header().chain_id.clone();

            validate_initial_signed_header_and_valset(&header, &validator_set).map_err(|e| {
              error!("Validation Error: {}", e);
              Error::<T>::ValidationError
            })?;

            let state: ConsensusState = ConsensusState{
                state: 	TrustedState::new(header.clone(), validator_set),
                last_update: Utc::now(),
            };

            let tmclient: TendermintClient = TendermintClient{
                state: Some(state.clone()),
                trusting_period: init_client_payload.trusting_period,
                client_id: init_client_payload.client_id.as_bytes().to_vec(),
                max_clock_drift: init_client_payload.max_clock_drift,
                unbonding_period: init_client_payload.unbonding_period,
                chain_id: chain_id.as_str().as_bytes().to_vec(),
                trust_threshold: TrustThresholdFraction::default(),
            };

            debug!("Storing newly created client: {:#?}", tmclient);

            TMClientStorage::insert(init_client_payload.client_id.as_bytes().to_vec(), TMClientStorageWrapper{client: tmclient.clone()});
            ClientInfoMap::insert(init_client_payload.client_id.as_bytes().to_vec(), TMClientInfo{
                chain_id: tmclient.chain_id.clone(),
                trusting_period: tmclient.trusting_period,
                max_clock_drift: tmclient.max_clock_drift,
                unbonding_period: tmclient.unbonding_period,
                last_block: header.header().height.value()
            });
            let mut available_clients = AvailableClients::get();
            available_clients.insert(available_clients.len(), init_client_payload.client_id.as_bytes().to_vec());
            AvailableClients::put(available_clients);

            // Here we are raising the ClientCreated event
            Self::deposit_event(RawEvent::ClientCreated(signer, tmclient.client_id, tmclient.chain_id, header.header().height.value()));
            Ok(())
        }

        /// Client initialisation entry point.
        /// takes json encoded `TMUpdateClientPayload` struct.
        #[weight = 100_000]
        pub fn update_client(origin, payload: Vec<u8>) -> dispatch::DispatchResult {
            // Check it was signed
            let signer = ensure_signed(origin)?;

            debug!("Submitted update client payload: {:?}", payload);

            let update_client_payload: TMUpdateClientPayload = serde_json::from_slice(&payload[..]).map_err(|e| {
              error!("Deserialization Error: {}", e);
              Error::<T>::DeserializeError
            })?;

            ensure!(TMClientStorage::contains_key(update_client_payload.client_id.as_bytes().to_vec()), Error::<T>::ItemNotFound);

            let mut wrapped_client: TMClientStorageWrapper = TMClientStorage::get(update_client_payload.client_id.as_bytes().to_vec());
            debug!("Fetched existing client from storage: {:#?}", wrapped_client);

            let header: LightSignedHeader = update_client_payload.header.signed_header;
            let validator_set: LightValidatorSet<LightValidator> = update_client_payload.header.validator_set;

            let trusted_state = verify_single(
                wrapped_client.client.state.unwrap().state.clone(),
                &header,
                &validator_set,
                &update_client_payload.next_validator_set,
                wrapped_client.client.trust_threshold,
                Duration::from_secs(wrapped_client.client.trusting_period+wrapped_client.client.max_clock_drift),
                std::time::SystemTime::now(),
            ).map_err(|e| {
                error!("Unable to validate header: {}", e);
                Error::<T>::ValidationError
            })?;

            let state: ConsensusState = ConsensusState{
                state: trusted_state,
                last_update: Utc::now(),
            };

            wrapped_client.client.state = Some(state.clone());
            TMClientStorage::insert(update_client_payload.client_id.as_bytes().to_vec(), wrapped_client.clone());
            debug!("Stored updated client in storage: {:#?}", wrapped_client);

            ClientInfoMap::insert(update_client_payload.client_id.as_bytes().to_vec(), TMClientInfo{
                chain_id: wrapped_client.client.chain_id.clone(),
                trusting_period: wrapped_client.client.trusting_period,
                max_clock_drift: wrapped_client.client.max_clock_drift,
                unbonding_period: wrapped_client.client.unbonding_period,
                last_block: header.header().height.value()
            });

            Self::deposit_event(RawEvent::ClientUpdated(signer, wrapped_client.client.client_id, wrapped_client.client.chain_id, header.header().height.value()));
            Ok(())
        }
    }
}

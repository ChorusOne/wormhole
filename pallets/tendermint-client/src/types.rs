use codec::{Decode, Encode, EncodeLike, Error, Input, Output};
use serde::{Deserialize, Serialize};
use sp_std::{default::Default, vec::Vec};

use chrono::{DateTime, Duration, Utc};
use tendermint_light_client::{
    Commit, LightHeader, LightSignedHeader, LightValidatorSet, Time, TrustThresholdFraction,
    TrustedState,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMHeader {
    pub signed_header: LightSignedHeader,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMCreateClientPayload {
    pub header: TMHeader,
    pub validator_set: LightValidatorSet,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    pub unbonding_period: u64,
    pub client_id: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMUpdateClientPayload {
    pub header: TMHeader,
    pub client_id: Vec<u8>,
    pub validator_set: LightValidatorSet,
    pub next_validator_set: LightValidatorSet,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConsensusState {
    pub state: TrustedState<Commit, LightHeader>,
    pub last_update: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TendermintClient {
    pub state: Option<ConsensusState>,
    pub client_id: Vec<u8>,
    pub chain_id: Vec<u8>,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    pub unbonding_period: u64,
    pub trust_threshold: TrustThresholdFraction,
    //pub owner: cosmosAddress,
}

impl Default for TendermintClient {
    fn default() -> Self {
        TendermintClient {
            state: None,
            client_id: Vec::new(),
            chain_id: Vec::new(),
            trusting_period: 86400,
            max_clock_drift: 30,
            unbonding_period: 86400 * 7 * 3,
            trust_threshold: TrustThresholdFraction::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TMClientStorageWrapper {
    pub client: TendermintClient,
}

impl Encode for TMClientStorageWrapper {
    fn encode_to<W: Output>(&self, dest: &mut W) {
        let json: Vec<u8> = serde_json::to_vec(&self.client).ok().unwrap();
        dest.write(&json[..]);
    }
}

impl Decode for TMClientStorageWrapper {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let bytes: Vec<u8> = Vec::decode(input)?;
        Ok(TMClientStorageWrapper {
            client: serde_json::from_slice(&bytes[..]).ok().unwrap(),
        })
    }
}

impl EncodeLike for TMClientStorageWrapper {}

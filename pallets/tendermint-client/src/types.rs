use codec::{Decode, Encode, EncodeLike, Error, Input, Output};
use serde::{Deserialize, Serialize};
use sp_std::{default::Default, vec::Vec};

use chrono::{DateTime, Utc};
use tendermint_light_client::{
    ClientId, Commit, LightHeader, LightSignedHeader, LightValidator, LightValidatorSet,
    TrustThresholdFraction, TrustedState,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMHeader {
    pub signed_header: LightSignedHeader,
    pub validator_set: LightValidatorSet<LightValidator>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMCreateClientPayload {
    pub header: TMHeader,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    pub unbonding_period: u64,
    pub client_id: ClientId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMUpdateClientPayload {
    pub header: TMHeader,
    pub client_id: ClientId,
    pub next_validator_set: LightValidatorSet<LightValidator>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConsensusState {
    pub state: TrustedState<Commit, LightHeader, LightValidator>,
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

#[derive(Serialize, Deserialize, Clone, Debug, Default, Encode, Decode)]
pub struct TMClientInfo {
    pub chain_id: Vec<u8>,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    pub unbonding_period: u64,
    pub last_block: u64,
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

#[allow(deprecated)]
impl Decode for TMClientStorageWrapper {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let len = input.remaining_len().unwrap().ok_or_else(|| "meh")?;
        let mut vec: Vec<u8> = Vec::with_capacity(len);
        vec.resize_default(len);
        let buf = vec.as_mut_slice();
        input.read(buf)?;
        Ok(TMClientStorageWrapper {
            client: serde_json::from_slice(&vec[..]).ok().unwrap(),
        })
    }
}

impl EncodeLike for TMClientStorageWrapper {}

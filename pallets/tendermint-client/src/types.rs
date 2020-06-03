use serde::{Serialize, Deserialize};
use codec::{Encode, Decode, Input, Output, Error, EncodeLike};
use std::{time::Duration, default::Default};
use tendermint::{
    block::{
        signed_header::SignedHeader,
    },
    time::Time,
    //lite::TrustThresholdFraction,
    validator::Set as ValidatorSet,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMCreateClientPayload {
    pub header: SignedHeader,
    pub trust_period: String,
    pub client_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TMUpdateClientPayload {
    pub header: SignedHeader,
    pub client_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConsensusState {
    pub signed_header: SignedHeader,
    pub next_validator_set: ValidatorSet,
    pub height: u64,
    pub last_update: Time,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TendermintClient {
    pub state: Option<ConsensusState>,
    pub client_name: String,
    pub chain_id: String,
    pub trusting_period: Duration,
}

impl Default for TendermintClient {
    fn default() -> Self {
        TendermintClient {
            state: None,
            client_name: String::default(),
            chain_id: String::default(),
            trusting_period: Duration::from_secs(0),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TMClientStorageWrapper {
    pub client: TendermintClient
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
        Ok(TMClientStorageWrapper{client: serde_json::from_slice(&bytes[..]).ok().unwrap()})
    }
}

impl EncodeLike for TMClientStorageWrapper {}
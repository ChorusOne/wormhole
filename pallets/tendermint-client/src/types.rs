use serde::Deserialize;
use std::time::Duration;
use tendermint::{
    block::{
        signed_header::SignedHeader,
    },
    time::Time,
    //lite::TrustThresholdFraction,
    validator::Set as ValidatorSet,
};

#[derive(Deserialize, Clone, Debug)]
pub struct TMCreateClientPayload {
    pub header: SignedHeader,
    pub trust_period: Duration,
    pub client_name: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TMUpdateClientPayload {
    pub header: SignedHeader,
    pub client_name: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ConsensusState {
    pub signed_header: SignedHeader,
    pub next_validator_set: ValidatorSet,
    pub height: u32,
    pub last_update: Time,
}

pub struct TendermintClient {
    pub state: ConsensusState,
    pub name: String,
    pub chain_id: String,
    pub trusting_period: Duration,
}
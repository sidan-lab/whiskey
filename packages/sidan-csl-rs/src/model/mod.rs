mod action;
mod asset;
mod data;
mod js_vec;
mod protocol;
mod serialized_address;
mod value;
pub use action::*;
pub use asset::*;
pub use data::*;
pub use js_vec::*;
pub use protocol::*;
use serde::{Deserialize, Serialize};
pub use serialized_address::*;
pub use value::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshTxBuilderBody {
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<Output>,
    pub collaterals: Vec<PubKeyTxIn>,
    pub required_signatures: JsVecString,
    pub reference_inputs: Vec<RefTxIn>,
    pub withdrawals: Vec<Withdrawal>,
    pub mints: Vec<MintItem>,
    pub change_address: String,
    pub change_datum: Option<Datum>,
    pub metadata: Vec<Metadata>,
    pub validity_range: ValidityRange,
    pub certificates: Vec<Certificate>,
    pub signing_key: JsVecString,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub address: String,
    pub amount: Vec<Asset>,
    pub datum: Option<Datum>,
    pub reference_script: Option<ProvidedScriptSource>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidityRange {
    pub invalid_before: Option<u64>,
    pub invalid_hereafter: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TxIn {
    PubKeyTxIn(PubKeyTxIn),
    SimpleScriptTxIn(SimpleScriptTxIn),
    ScriptTxIn(ScriptTxIn),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefTxIn {
    pub tx_hash: String,
    pub tx_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PubKeyTxIn {
    pub tx_in: TxInParameter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleScriptTxIn {
    pub tx_in: TxInParameter,
    pub simple_script_tx_in: SimpleScriptTxInParameter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimpleScriptTxInParameter {
    ProvidedSimpleScriptSource(ProvidedSimpleScriptSource),
    InlineSimpleScriptSource(InlineSimpleScriptSource),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidedSimpleScriptSource {
    pub script_cbor: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineSimpleScriptSource {
    pub ref_tx_in: RefTxIn,
    pub simple_script_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTxIn {
    pub tx_in: TxInParameter,
    pub script_tx_in: ScriptTxInParameter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInParameter {
    pub tx_hash: String,
    pub tx_index: u32,
    pub amount: Option<Vec<Asset>>,
    pub address: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTxInParameter {
    pub script_source: Option<ScriptSource>,
    pub datum_source: Option<DatumSource>,
    pub redeemer: Option<Redeemer>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScriptSource {
    ProvidedScriptSource(ProvidedScriptSource),
    InlineScriptSource(InlineScriptSource),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidedScriptSource {
    pub script_cbor: String,
    pub language_version: LanguageVersion,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineScriptSource {
    pub tx_hash: String,
    pub tx_index: u32,
    pub spending_script_hash: String,
    pub language_version: LanguageVersion,
    pub script_size: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LanguageVersion {
    V1,
    V2,
    V3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DatumSource {
    ProvidedDatumSource(ProvidedDatumSource),
    InlineDatumSource(InlineDatumSource),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidedDatumSource {
    pub data: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineDatumSource {
    pub tx_hash: String,
    pub tx_index: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptSourceInfo {
    pub tx_hash: String,
    pub tx_index: u32,
    pub spending_script_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Withdrawal {
    PubKeyWithdrawal(PubKeyWithdrawal),
    PlutusScriptWithdrawal(PlutusScriptWithdrawal),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PubKeyWithdrawal {
    pub address: String,
    pub coin: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlutusScriptWithdrawal {
    pub address: String,
    pub coin: u64,
    pub script_source: Option<ScriptSource>,
    pub redeemer: Option<Redeemer>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintItem {
    pub type_: MintItemType,
    pub policy_id: String,
    pub asset_name: String,
    pub amount: u64,
    pub redeemer: Option<Redeemer>,
    pub script_source: Option<ScriptSource>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MintItemType {
    Native,
    Plutus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Redeemer {
    pub data: String,
    pub ex_units: Budget,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Budget {
    pub mem: u64,
    pub steps: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub tag: String,
    pub metadata: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Certificate {
    RegisterStake(RegisterStake),
    DeregisterStake(DeregisterStake),
    DelegateStake(DelegateStake),
    RegisterPool(RegisterPool),
    RetirePool(RetirePool),
    VoteDelegation(VoteDelegation),
    StakeAndVoteDelegation(StakeAndVoteDelegation),
    StakeRegistrationAndDelegation(StakeRegistrationAndDelegation),
    VoteRegistrationAndDelegation(VoteRegistrationAndDelegation),
    StakeVoteRegistrationAndDelegation(StakeVoteRegistrationAndDelegation),
    CommitteeHotAuth(CommitteeHotAuth),
    CommitteeColdResign(CommitteeColdResign),
    DRepRegistration(DRepRegistration),
    DRepDeregistration(DRepDeregistration),
    DRepUpdate(DRepUpdate),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterPool {
    pub pool_params: PoolParams,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolParams {
    pub vrf_key_hash: String,
    pub operator: String,
    pub pledge: String,
    pub cost: String,
    pub margin: (u64, u64),
    pub relays: Vec<Relay>,
    pub owners: Vec<String>,
    pub reward_address: String,
    pub metadata: Option<PoolMetadata>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Relay {
    SingleHostAddr(SingleHostAddr),
    SingleHostName(SingleHostName),
    MultiHostName(MultiHostName),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleHostAddr {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleHostName {
    pub domain_name: String,
    pub port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiHostName {
    pub domain_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMetadata {
    pub url: String,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterStake {
    pub stake_key_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegateStake {
    pub stake_key_hash: String,
    pub pool_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeregisterStake {
    pub stake_key_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetirePool {
    pub pool_id: String,
    pub epoch: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteDelegation {
    pub stake_key_hash: String,
    pub drep: DRep,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DRep {
    KeyHash(String),
    ScriptHash(String),
    AlwaysAbstain,
    AlwaysNoConfidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakeAndVoteDelegation {
    pub stake_key_hash: String,
    pub pool_key_hash: String,
    pub drep: DRep,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakeRegistrationAndDelegation {
    pub stake_key_hash: String,
    pub pool_key_hash: String,
    pub coin: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteRegistrationAndDelegation {
    pub stake_key_hash: String,
    pub drep: DRep,
    pub coin: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakeVoteRegistrationAndDelegation {
    pub stake_key_hash: String,
    pub pool_key_hash: String,
    pub drep: DRep,
    pub coin: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitteeHotAuth {
    pub committee_cold_key_hash: String,
    pub committee_hot_key_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitteeColdResign {
    pub committee_cold_key_hash: String,
    pub anchor: Option<Anchor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    pub anchor_url: String,
    pub anchor_data_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DRepRegistration {
    pub voting_key_hash: String,
    pub coin: u64,
    pub anchor: Option<Anchor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DRepDeregistration {
    pub voting_key_hash: String,
    pub coin: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DRepUpdate {
    pub voting_key_hash: String,
    pub anchor: Option<Anchor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Datum {
    Inline(String),
    Hash(String),
}

impl Datum {
    pub fn get_inner(&self) -> &str {
        match self {
            Datum::Inline(s) => s,
            Datum::Hash(s) => s,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtxoInput {
    pub output_index: u32,
    pub tx_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtxoOutput {
    pub address: String,
    pub amount: Vec<Asset>,
    pub data_hash: Option<String>,
    pub plutus_data: Option<String>,
    pub script_ref: Option<String>,
    pub script_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UTxO {
    pub input: UtxoInput,
    pub output: UtxoOutput,
}

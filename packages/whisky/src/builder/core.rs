use async_trait::async_trait;
use cardano_serialization_lib::JsError;
use sidan_csl_rs::{
    builder::{serialize_tx_body, IMeshTxBuilderCore, MeshTxBuilderCore},
    core::{algo::select_utxos, builder::IMeshCSL, utils::build_tx_builder},
    csl,
    model::{
        Anchor, Asset, Certificate, CommitteeColdResign, CommitteeHotAuth, DRep,
        DRepDeregistration, DRepRegistration, DRepUpdate, Datum, DatumSource, DelegateStake,
        DeregisterStake, InlineDatumSource, InlineScriptSource, LanguageVersion, MeshTxBuilderBody,
        Metadata, MintItem, MintItemType, Output, PlutusScriptWithdrawal, PoolParams,
        ProvidedDatumSource, ProvidedScriptSource, ProvidedSimpleScriptSource, PubKeyTxIn,
        PubKeyWithdrawal, Redeemer, RefTxIn, RegisterPool, RegisterStake, RetirePool, ScriptSource,
        ScriptTxIn, ScriptTxInParameter, SimpleScriptTxIn, SimpleScriptTxInParameter,
        StakeAndVoteDelegation, StakeRegistrationAndDelegation, StakeVoteRegistrationAndDelegation,
        TxIn, TxInParameter, UTxO, Value, VoteDelegation, VoteRegistrationAndDelegation,
        Withdrawal,
    },
};

use super::{IMeshTxBuilder, MeshTxBuilder, MeshTxEvaluator, WData, WRedeemer};
use crate::service::ITxEvaluation;

#[async_trait]
impl IMeshTxBuilder for MeshTxBuilder {
    fn new(param: super::MeshTxBuilderParam) -> Self {
        MeshTxBuilder {
            core: MeshTxBuilderCore::new_core(None),
            protocol_params: param.params,
            tx_in_item: None,
            withdrawal_item: None,
            mint_item: None,
            collateral_item: None,
            tx_output: None,
            adding_script_input: false,
            adding_plutus_mint: false,
            adding_plutus_withdrawal: false,
            fetcher: param.fetcher,
            evaluator: match param.evaluator {
                Some(evaluator) => Some(evaluator),
                None => Some(Box::new(MeshTxEvaluator::new())),
            },
            submitter: param.submitter,
            extra_inputs: vec![],
            selection_threshold: 5_000_000,
            chained_txs: vec![],
            inputs_for_evaluation: vec![],
        }
    }

    fn new_core() -> Self {
        Self::new(super::MeshTxBuilderParam {
            evaluator: None,
            fetcher: None,
            submitter: None,
            params: None,
        })
    }

    async fn complete(
        &mut self,
        customized_tx: Option<MeshTxBuilderBody>,
    ) -> Result<&mut Self, JsError> {
        self.complete_sync(customized_tx)?;
        match &self.evaluator {
            Some(evaluator) => {
                let tx_evaluation_result = evaluator
                    .evaluate_tx(
                        &self.core.mesh_csl.tx_hex,
                        &self.inputs_for_evaluation.clone(),
                        &self.chained_txs.clone(),
                    )
                    .await;
                match tx_evaluation_result {
                    Ok(actions) => self.update_redeemer(actions),
                    Err(_) => panic!("Error evaluating transaction"),
                }
            }
            None => self,
        };
        self.complete_sync(None)
    }

    fn complete_sync(
        &mut self,
        customized_tx: Option<MeshTxBuilderBody>,
    ) -> Result<&mut Self, JsError> {
        if customized_tx.is_some() {
            self.core.mesh_tx_builder_body = customized_tx.unwrap();
        } else {
            self.queue_all_last_item();
            if !self.extra_inputs.is_empty() {
                self.add_utxos_from(self.extra_inputs.clone(), self.selection_threshold)?;
            }
        }

        self.core
            .mesh_tx_builder_body
            .mints
            .sort_by(|a, b| a.policy_id.cmp(&b.policy_id));

        self.core.mesh_tx_builder_body.inputs.sort_by(|a, b| {
            let tx_in_data_a: &TxInParameter = match a {
                TxIn::PubKeyTxIn(pub_key_tx_in) => &pub_key_tx_in.tx_in,
                TxIn::SimpleScriptTxIn(simple_script_tx_in) => &simple_script_tx_in.tx_in,
                TxIn::ScriptTxIn(script_tx_in) => &script_tx_in.tx_in,
            };

            let tx_in_data_b: &TxInParameter = match b {
                TxIn::PubKeyTxIn(pub_key_tx_in) => &pub_key_tx_in.tx_in,
                TxIn::SimpleScriptTxIn(simple_script_tx_in) => &simple_script_tx_in.tx_in,
                TxIn::ScriptTxIn(script_tx_in) => &script_tx_in.tx_in,
            };

            tx_in_data_a
                .tx_hash
                .cmp(&tx_in_data_b.tx_hash)
                .then_with(|| tx_in_data_a.tx_index.cmp(&tx_in_data_b.tx_index))
        });

        let tx_hex = serialize_tx_body(
            self.core.mesh_tx_builder_body.clone(),
            self.protocol_params.clone(),
        )?;
        self.core.mesh_csl.tx_hex = tx_hex;
        self.core.mesh_csl.tx_builder = build_tx_builder(None);
        self.core.mesh_csl.tx_inputs_builder = csl::TxInputsBuilder::new();
        Ok(self)
    }

    fn complete_signing(&mut self) -> String {
        self.core.complete_signing()
    }

    fn tx_hex(&mut self) -> String {
        self.core.mesh_csl.tx_hex.to_string()
    }

    fn tx_in(
        &mut self,
        tx_hash: &str,
        tx_index: u32,
        amount: Vec<Asset>,
        address: &str,
    ) -> &mut Self {
        if self.tx_in_item.is_some() {
            self.queue_input();
        }
        if !self.adding_script_input {
            let item = TxIn::PubKeyTxIn(PubKeyTxIn {
                tx_in: TxInParameter {
                    tx_hash: tx_hash.to_string(),
                    tx_index,
                    amount: Some(amount),
                    address: Some(address.to_string()),
                },
            });
            self.tx_in_item = Some(item);
        } else {
            let item = TxIn::ScriptTxIn(ScriptTxIn {
                tx_in: TxInParameter {
                    tx_hash: tx_hash.to_string(),
                    tx_index,
                    amount: Some(amount),
                    address: Some(address.to_string()),
                },
                script_tx_in: ScriptTxInParameter {
                    script_source: None,
                    datum_source: None,
                    redeemer: None,
                },
            });
            self.tx_in_item = Some(item);
        }
        self
    }

    fn tx_in_script(&mut self, script_cbor: &str, version: Option<LanguageVersion>) -> &mut Self {
        let tx_in_item = self.tx_in_item.take();
        if tx_in_item.is_none() {
            panic!("Undefined input")
        }
        let tx_in_item = tx_in_item.unwrap();
        match tx_in_item {
            TxIn::PubKeyTxIn(input) => {
                self.tx_in_item = Some(TxIn::SimpleScriptTxIn(SimpleScriptTxIn {
                    tx_in: input.tx_in,
                    simple_script_tx_in: SimpleScriptTxInParameter::ProvidedSimpleScriptSource(
                        ProvidedSimpleScriptSource {
                            script_cbor: script_cbor.to_string(),
                        },
                    ),
                }))
            }
            TxIn::SimpleScriptTxIn(mut input) => {
                input.simple_script_tx_in = SimpleScriptTxInParameter::ProvidedSimpleScriptSource(
                    ProvidedSimpleScriptSource {
                        script_cbor: script_cbor.to_string(),
                    },
                );
                self.tx_in_item = Some(TxIn::SimpleScriptTxIn(input));
            }
            TxIn::ScriptTxIn(mut input) => {
                input.script_tx_in.script_source =
                    Some(ScriptSource::ProvidedScriptSource(ProvidedScriptSource {
                        script_cbor: script_cbor.to_string(),
                        language_version: version.unwrap(),
                    }));
                self.tx_in_item = Some(TxIn::ScriptTxIn(input));
            }
        }
        self
    }

    fn tx_in_datum_value(&mut self, data: WData) -> &mut Self {
        let tx_in_item = self.tx_in_item.take();
        if tx_in_item.is_none() {
            panic!("Undefined input")
        }
        let tx_in_item = tx_in_item.unwrap();
        match tx_in_item {
            TxIn::PubKeyTxIn(_) => panic!("Datum cannot be defined for a pubkey tx in"),
            TxIn::SimpleScriptTxIn(_) => {
                panic!("Datum cannot be defined for a simple script tx in")
            }
            TxIn::ScriptTxIn(mut input) => match data.to_cbor() {
                Ok(raw_data) => {
                    input.script_tx_in.datum_source =
                        Some(DatumSource::ProvidedDatumSource(ProvidedDatumSource {
                            data: raw_data.to_string(),
                        }));
                    self.tx_in_item = Some(TxIn::ScriptTxIn(input));
                }
                Err(_) => {
                    panic!("Error converting datum to CBOR");
                }
            },
        }
        self
    }

    fn tx_in_inline_datum_present(&mut self) -> &mut Self {
        let tx_in_item = self.tx_in_item.take();
        if tx_in_item.is_none() {
            panic!("Undefined input")
        }
        let tx_in_item = tx_in_item.unwrap();
        match tx_in_item {
            TxIn::PubKeyTxIn(_) => panic!("Datum cannot be defined for a pubkey tx in"),
            TxIn::SimpleScriptTxIn(_) => {
                panic!("Datum cannot be defined for a simple script tx in")
            }
            TxIn::ScriptTxIn(mut input) => {
                input.script_tx_in.datum_source =
                    Some(DatumSource::InlineDatumSource(InlineDatumSource {
                        tx_hash: input.tx_in.tx_hash.clone(),
                        tx_index: input.tx_in.tx_index,
                    }));
                self.tx_in_item = Some(TxIn::ScriptTxIn(input));
            }
        }
        self
    }

    fn tx_in_redeemer_value(&mut self, redeemer: WRedeemer) -> &mut Self {
        let tx_in_item = self.tx_in_item.take();
        if tx_in_item.is_none() {
            panic!("Undefined input")
        }
        let tx_in_item = tx_in_item.unwrap();
        match tx_in_item {
            TxIn::PubKeyTxIn(_) => panic!("Redeemer cannot be defined for a pubkey tx in"),
            TxIn::SimpleScriptTxIn(_) => {
                panic!("Redeemer cannot be defined for a simple script tx in")
            }
            TxIn::ScriptTxIn(mut input) => match redeemer.data.to_cbor() {
                Ok(raw_redeemer) => {
                    input.script_tx_in.redeemer = Some(Redeemer {
                        data: raw_redeemer,
                        ex_units: redeemer.ex_units,
                    });
                    self.tx_in_item = Some(TxIn::ScriptTxIn(input));
                }
                Err(_) => {
                    panic!("Error converting redeemer to CBOR");
                }
            },
        }
        self
    }

    fn tx_out(&mut self, address: &str, amount: Vec<Asset>) -> &mut Self {
        if self.tx_output.is_some() {
            let tx_output = self.tx_output.take();
            self.core
                .mesh_tx_builder_body
                .outputs
                .push(tx_output.unwrap());
        }
        self.tx_output = Some(Output {
            address: address.to_string(),
            amount,
            datum: None,
            reference_script: None,
        });
        self
    }

    fn tx_out_datum_hash_value(&mut self, data: WData) -> &mut Self {
        let tx_output = self.tx_output.take();
        if tx_output.is_none() {
            panic!("Undefined output")
        }
        let mut tx_output = tx_output.unwrap();
        match data.to_cbor() {
            Ok(raw_data) => {
                tx_output.datum = Some(Datum::Hash(raw_data));
                self.tx_output = Some(tx_output);
            }
            Err(_) => {
                panic!("Error converting datum to CBOR");
            }
        }
        self
    }

    fn tx_out_inline_datum_value(&mut self, data: WData) -> &mut Self {
        let tx_output = self.tx_output.take();
        if tx_output.is_none() {
            panic!("Undefined output")
        }
        let mut tx_output = tx_output.unwrap();
        match data.to_cbor() {
            Ok(raw_data) => {
                tx_output.datum = Some(Datum::Inline(raw_data));
                self.tx_output = Some(tx_output);
            }
            Err(_) => {
                panic!("Error converting datum to CBOR");
            }
        }
        self
    }

    fn tx_out_reference_script(
        &mut self,
        script_cbor: &str,
        version: LanguageVersion,
    ) -> &mut Self {
        let tx_output = self.tx_output.take();
        if tx_output.is_none() {
            panic!("Undefined output")
        }
        let mut tx_output = tx_output.unwrap();
        tx_output.reference_script = Some(ProvidedScriptSource {
            script_cbor: script_cbor.to_string(),
            language_version: version,
        });
        self.tx_output = Some(tx_output);
        self
    }

    fn spending_plutus_script_v2(&mut self) -> &mut Self {
        self.adding_script_input = true;
        self
    }

    fn spending_tx_in_reference(
        &mut self,
        tx_hash: &str,
        tx_index: u32,
        spending_script_hash: &str,
        version: LanguageVersion,
        script_size: usize,
    ) -> &mut Self {
        let tx_in_item = self.tx_in_item.take();
        if tx_in_item.is_none() {
            panic!("Undefined output")
        }
        let tx_in_item = tx_in_item.unwrap();
        match tx_in_item {
            TxIn::PubKeyTxIn(_) => panic!("Script reference cannot be defined for a pubkey tx in"),
            TxIn::SimpleScriptTxIn(_) => {
                panic!("Script reference cannot be defined for a simple script tx in")
            }
            TxIn::ScriptTxIn(mut input) => {
                input.script_tx_in.script_source =
                    Some(ScriptSource::InlineScriptSource(InlineScriptSource {
                        tx_hash: tx_hash.to_string(),
                        tx_index,
                        spending_script_hash: spending_script_hash.to_string(),
                        language_version: version,
                        script_size,
                    }));
                self.tx_in_item = Some(TxIn::ScriptTxIn(input));
            }
        }
        self
    }

    fn spending_reference_tx_in_inline_datum_present(&mut self) -> &mut Self {
        self.tx_in_inline_datum_present()
    }

    fn spending_reference_tx_in_redeemer_value(&mut self, redeemer: WRedeemer) -> &mut Self {
        self.tx_in_redeemer_value(redeemer)
    }

    fn read_only_tx_in_reference(&mut self, tx_hash: &str, tx_index: u32) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .reference_inputs
            .push(RefTxIn {
                tx_hash: tx_hash.to_string(),
                tx_index,
            });
        self
    }

    fn withdrawal_plutus_script_v2(&mut self) -> &mut Self {
        self.adding_plutus_withdrawal = true;
        self
    }

    fn withdrawal_tx_in_reference(
        &mut self,
        tx_hash: &str,
        tx_index: u32,
        withdrawal_script_hash: &str,
        version: LanguageVersion,
        script_size: usize,
    ) -> &mut Self {
        let withdrawal_item = self.withdrawal_item.take();
        if withdrawal_item.is_none() {
            panic!("Undefined output")
        }
        let withdrawal_item = withdrawal_item.unwrap();
        match withdrawal_item {
            Withdrawal::PubKeyWithdrawal(_) => {
                panic!("Script reference cannot be defined for a pubkey withdrawal")
            }
            Withdrawal::PlutusScriptWithdrawal(mut withdrawal) => {
                withdrawal.script_source =
                    Some(ScriptSource::InlineScriptSource(InlineScriptSource {
                        tx_hash: tx_hash.to_string(),
                        tx_index,
                        spending_script_hash: withdrawal_script_hash.to_string(),
                        language_version: version,
                        script_size,
                    }));
                self.withdrawal_item = Some(Withdrawal::PlutusScriptWithdrawal(withdrawal));
            }
        }
        self
    }

    fn withdrawal(&mut self, stake_address: &str, coin: u64) -> &mut Self {
        if self.withdrawal_item.is_some() {
            self.queue_withdrawal();
        }
        if !self.adding_plutus_withdrawal {
            let withdrawal_item = Withdrawal::PubKeyWithdrawal(PubKeyWithdrawal {
                address: stake_address.to_string(),
                coin,
            });
            self.withdrawal_item = Some(withdrawal_item);
        } else {
            let withdrawal_item = Withdrawal::PlutusScriptWithdrawal(PlutusScriptWithdrawal {
                address: stake_address.to_string(),
                coin,
                script_source: None,
                redeemer: None,
            });
            self.withdrawal_item = Some(withdrawal_item);
        }
        self.adding_plutus_withdrawal = false;
        self
    }

    fn withdrawal_script(&mut self, script_cbor: &str, version: LanguageVersion) -> &mut Self {
        let withdrawal_item = self.withdrawal_item.take();
        if withdrawal_item.is_none() {
            panic!("Undefined withdrawal")
        }
        let withdrawal_item = withdrawal_item.unwrap();
        match withdrawal_item {
            Withdrawal::PubKeyWithdrawal(_) => {
                panic!("Script cannot be defined for a pubkey withdrawal")
            }
            Withdrawal::PlutusScriptWithdrawal(mut withdraw) => {
                withdraw.script_source =
                    Some(ScriptSource::ProvidedScriptSource(ProvidedScriptSource {
                        script_cbor: script_cbor.to_string(),
                        language_version: version,
                    }));
                self.withdrawal_item = Some(Withdrawal::PlutusScriptWithdrawal(withdraw));
            }
        }
        self
    }

    fn withdrawal_redeemer_value(&mut self, redeemer: WRedeemer) -> &mut Self {
        let withdrawal_item = self.withdrawal_item.take();
        if withdrawal_item.is_none() {
            panic!("Undefined input")
        }
        let withdrawal_item = withdrawal_item.unwrap();
        match withdrawal_item {
            Withdrawal::PubKeyWithdrawal(_) => {
                panic!("Script cannot be defined for a pubkey withdrawal")
            }
            Withdrawal::PlutusScriptWithdrawal(mut withdraw) => match redeemer.data.to_cbor() {
                Ok(raw_redeemer) => {
                    withdraw.redeemer = Some(Redeemer {
                        data: raw_redeemer,
                        ex_units: redeemer.ex_units,
                    });
                    self.withdrawal_item = Some(Withdrawal::PlutusScriptWithdrawal(withdraw));
                }
                Err(_) => panic!("Error converting redeemer to CBOR"),
            },
        }
        self
    }

    fn withdrawal_reference_tx_in_redeemer_value(&mut self, redeemer: WRedeemer) -> &mut Self {
        self.withdrawal_redeemer_value(redeemer)
    }

    fn mint_plutus_script_v2(&mut self) -> &mut Self {
        self.adding_plutus_mint = true;
        self
    }

    fn mint(&mut self, quantity: u64, policy: &str, name: &str) -> &mut Self {
        if self.mint_item.is_some() {
            self.queue_mint();
        }
        let mint_type = if self.adding_plutus_mint {
            MintItemType::Plutus
        } else {
            MintItemType::Native
        };
        self.mint_item = Some(MintItem {
            type_: mint_type,
            policy_id: policy.to_string(),
            asset_name: name.to_string(),
            amount: quantity,
            redeemer: None,
            script_source: None,
        });
        self.adding_plutus_mint = false;
        self
    }

    fn minting_script(&mut self, script_cbor: &str, version: LanguageVersion) -> &mut Self {
        let mint_item = self.mint_item.take();
        if mint_item.is_none() {
            panic!("Undefined mint");
        }
        let mut mint_item = mint_item.unwrap();
        mint_item.script_source = Some(ScriptSource::ProvidedScriptSource(ProvidedScriptSource {
            script_cbor: script_cbor.to_string(),
            language_version: version,
        }));
        self.mint_item = Some(mint_item);
        self
    }

    fn mint_tx_in_reference(
        &mut self,
        tx_hash: &str,
        tx_index: u32,
        spending_script_hash: &str,
        version: LanguageVersion,
        script_size: usize,
    ) -> &mut Self {
        let mint_item = self.mint_item.take();
        if mint_item.is_none() {
            panic!("Undefined mint");
        }
        let mut mint_item = mint_item.unwrap();
        mint_item.script_source = Some(ScriptSource::InlineScriptSource(InlineScriptSource {
            tx_hash: tx_hash.to_string(),
            tx_index,
            spending_script_hash: spending_script_hash.to_string(),
            language_version: version,
            script_size,
        }));
        self.mint_item = Some(mint_item);
        self
    }

    fn mint_redeemer_value(&mut self, redeemer: WRedeemer) -> &mut Self {
        let mint_item = self.mint_item.take();
        if mint_item.is_none() {
            panic!("Undefined mint");
        }
        let mut mint_item = mint_item.unwrap();
        if mint_item.type_ == MintItemType::Native {
            panic!("Redeemer cannot be defined for Native script mints");
        }
        match redeemer.data.to_cbor() {
            Ok(raw_redeemer) => {
                mint_item.redeemer = Some(Redeemer {
                    data: raw_redeemer,
                    ex_units: redeemer.ex_units,
                });
                self.mint_item = Some(mint_item);
            }
            Err(_) => {
                panic!("Error converting redeemer to CBOR");
            }
        }
        self
    }

    fn mint_reference_tx_in_redeemer_value(&mut self, redeemer: WRedeemer) -> &mut Self {
        self.mint_redeemer_value(redeemer)
    }

    fn required_signer_hash(&mut self, pub_key_hash: &str) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .required_signatures
            .add(pub_key_hash.to_string());
        self
    }

    fn tx_in_collateral(
        &mut self,
        tx_hash: &str,
        tx_index: u32,
        amount: Vec<Asset>,
        address: &str,
    ) -> &mut Self {
        let collateral_item = self.collateral_item.take();
        if let Some(collateral_item) = collateral_item {
            self.core
                .mesh_tx_builder_body
                .collaterals
                .push(collateral_item);
        }
        self.collateral_item = Some(PubKeyTxIn {
            tx_in: TxInParameter {
                tx_hash: tx_hash.to_string(),
                tx_index,
                amount: Some(amount),
                address: Some(address.to_string()),
            },
        });
        self
    }

    fn register_pool_certificate(&mut self, pool_params: PoolParams) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::RegisterPool(RegisterPool { pool_params }));
        self
    }

    fn register_stake_certificate(&mut self, stake_key_hash: &str) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::RegisterStake(RegisterStake {
                stake_key_hash: stake_key_hash.to_string(),
            }));
        self
    }

    fn delegate_stake_certificate(&mut self, stake_key_hash: &str, pool_id: &str) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::DelegateStake(DelegateStake {
                stake_key_hash: stake_key_hash.to_string(),
                pool_id: pool_id.to_string(),
            }));
        self
    }

    fn deregister_stake_certificate(&mut self, stake_key_hash: &str) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::DeregisterStake(DeregisterStake {
                stake_key_hash: stake_key_hash.to_string(),
            }));
        self
    }

    fn retire_pool_certificate(&mut self, pool_id: &str, epoch: u32) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::RetirePool(RetirePool {
                pool_id: pool_id.to_string(),
                epoch,
            }));
        self
    }

    fn vote_delegation_certificate(&mut self, stake_key_hash: &str, drep: DRep) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::VoteDelegation(VoteDelegation {
                stake_key_hash: stake_key_hash.to_string(),
                drep,
            }));
        self
    }

    fn stake_and_vote_delegation_certificate(
        &mut self,
        stake_key_hash: &str,
        pool_key_hash: &str,
        drep: DRep,
    ) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::StakeAndVoteDelegation(
                StakeAndVoteDelegation {
                    stake_key_hash: stake_key_hash.to_string(),
                    pool_key_hash: pool_key_hash.to_string(),
                    drep,
                },
            ));
        self
    }

    fn stake_registration_and_delegation(
        &mut self,
        stake_key_hash: &str,
        pool_key_hash: &str,
        coin: u64,
    ) -> &mut Self {
        self.core.mesh_tx_builder_body.certificates.push(
            Certificate::StakeRegistrationAndDelegation(StakeRegistrationAndDelegation {
                stake_key_hash: stake_key_hash.to_string(),
                pool_key_hash: pool_key_hash.to_string(),
                coin,
            }),
        );
        self
    }

    fn vote_registration_and_delegation(
        &mut self,
        stake_key_hash: &str,
        drep: DRep,
        coin: u64,
    ) -> &mut Self {
        self.core.mesh_tx_builder_body.certificates.push(
            Certificate::VoteRegistrationAndDelegation(VoteRegistrationAndDelegation {
                stake_key_hash: stake_key_hash.to_string(),
                drep,
                coin,
            }),
        );
        self
    }

    fn stake_vote_registration_and_delegation(
        &mut self,
        stake_key_hash: &str,
        pool_key_hash: &str,
        drep: DRep,
        coin: u64,
    ) -> &mut Self {
        self.core.mesh_tx_builder_body.certificates.push(
            Certificate::StakeVoteRegistrationAndDelegation(StakeVoteRegistrationAndDelegation {
                stake_key_hash: stake_key_hash.to_string(),
                pool_key_hash: pool_key_hash.to_string(),
                drep,
                coin,
            }),
        );
        self
    }

    fn committee_hot_auth(
        &mut self,
        committee_cold_key_hash: &str,
        committee_hot_key_hash: &str,
    ) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::CommitteeHotAuth(CommitteeHotAuth {
                committee_cold_key_hash: committee_cold_key_hash.to_string(),
                committee_hot_key_hash: committee_hot_key_hash.to_string(),
            }));
        self
    }

    fn commitee_cold_resign(
        &mut self,
        committee_cold_key_hash: &str,
        anchor: Option<Anchor>,
    ) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::CommitteeColdResign(CommitteeColdResign {
                committee_cold_key_hash: committee_cold_key_hash.to_string(),
                anchor,
            }));
        self
    }

    fn drep_registration(
        &mut self,
        voting_key_hash: &str,
        coin: u64,
        anchor: Option<Anchor>,
    ) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::DRepRegistration(DRepRegistration {
                voting_key_hash: voting_key_hash.to_string(),
                coin,
                anchor,
            }));
        self
    }

    fn drep_deregistration(&mut self, voting_key_hash: &str, coin: u64) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::DRepDeregistration(DRepDeregistration {
                voting_key_hash: voting_key_hash.to_string(),
                coin,
            }));
        self
    }

    fn drep_update(&mut self, voting_key_hash: &str, anchor: Option<Anchor>) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .certificates
            .push(Certificate::DRepUpdate(DRepUpdate {
                voting_key_hash: voting_key_hash.to_string(),
                anchor,
            }));
        self
    }

    fn change_address(&mut self, address: &str) -> &mut Self {
        self.core.mesh_tx_builder_body.change_address = address.to_string();
        self
    }

    fn change_output_datum(&mut self, data: WData) -> &mut Self {
        match data.to_cbor() {
            Ok(raw_data) => {
                self.core.mesh_tx_builder_body.change_datum = Some(Datum::Inline(raw_data));
            }
            Err(_) => {
                panic!("Error converting datum to CBOR");
            }
        }
        self
    }

    fn invalid_before(&mut self, slot: u64) -> &mut Self {
        self.core.mesh_tx_builder_body.validity_range.invalid_before = Some(slot);
        self
    }

    fn invalid_hereafter(&mut self, slot: u64) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .validity_range
            .invalid_hereafter = Some(slot);
        self
    }

    fn metadata_value(&mut self, tag: &str, metadata: &str) -> &mut Self {
        self.core.mesh_tx_builder_body.metadata.push(Metadata {
            tag: tag.to_string(),
            metadata: metadata.to_string(),
        });
        self
    }

    fn signing_key(&mut self, skey_hex: &str) -> &mut Self {
        self.core
            .mesh_tx_builder_body
            .signing_key
            .add(skey_hex.to_string());
        self
    }

    fn chain_tx(&mut self, tx_hex: &str) -> &mut Self {
        self.chained_txs.push(tx_hex.to_string());
        self
    }

    fn input_for_evaluation(&mut self, input: UTxO) -> &mut Self {
        self.inputs_for_evaluation.push(input);
        self
    }

    fn select_utxos_from(&mut self, extra_inputs: Vec<UTxO>, threshold: u64) -> &mut Self {
        self.selection_threshold = threshold;
        self.extra_inputs = extra_inputs;
        self
    }

    fn queue_input(&mut self) {
        let tx_in_item = self.tx_in_item.clone().unwrap();
        match tx_in_item {
            TxIn::ScriptTxIn(tx_in) => {
                match (
                    tx_in.script_tx_in.datum_source,
                    tx_in.script_tx_in.redeemer,
                    tx_in.script_tx_in.script_source,
                ) {
                    (None, _, _) => panic!("Datum in a script input cannot be None"),
                    (_, None, _) => panic!("Redeemer in script input cannot be None"),
                    (_, _, None) => panic!("Script source in script input cannot be None"),
                    _ => {}
                }
            }
            TxIn::SimpleScriptTxIn(_) => {}
            TxIn::PubKeyTxIn(_) => {}
        }
        self.core
            .mesh_tx_builder_body
            .inputs
            .push(self.tx_in_item.clone().unwrap());
        self.tx_in_item = None
    }

    fn queue_withdrawal(&mut self) {
        let withdrawal_item = self.withdrawal_item.clone().unwrap();
        match withdrawal_item {
            Withdrawal::PlutusScriptWithdrawal(withdrawal) => {
                match (withdrawal.redeemer, withdrawal.script_source) {
                    (None, _) => panic!("Redeemer in script input cannot be None"),
                    (_, None) => panic!("Script source in script input cannot be None"),
                    _ => {}
                }
            }
            Withdrawal::PubKeyWithdrawal(_) => {}
        }
        self.core
            .mesh_tx_builder_body
            .withdrawals
            .push(self.withdrawal_item.clone().unwrap());
        self.withdrawal_item = None;
    }

    fn queue_mint(&mut self) {
        let mint_item = self.mint_item.clone().unwrap();
        if mint_item.script_source.is_none() {
            panic!("Missing mint script information");
        }
        self.core.mesh_tx_builder_body.mints.push(mint_item);
        self.mint_item = None;
    }

    fn queue_all_last_item(&mut self) {
        if self.tx_output.is_some() {
            self.core
                .mesh_tx_builder_body
                .outputs
                .push(self.tx_output.clone().unwrap());
            self.tx_output = None;
        }
        if self.tx_in_item.is_some() {
            self.queue_input();
        }
        if self.collateral_item.is_some() {
            self.core
                .mesh_tx_builder_body
                .collaterals
                .push(self.collateral_item.clone().unwrap());
            self.collateral_item = None;
        }
        if self.withdrawal_item.is_some() {
            self.queue_withdrawal();
        }
        if self.mint_item.is_some() {
            self.queue_mint();
        }
    }

    fn add_utxos_from(&mut self, extra_inputs: Vec<UTxO>, threshold: u64) -> Result<(), JsError> {
        let mut required_assets = Value::new();

        for output in &self.core.mesh_tx_builder_body.outputs {
            let output_value = Value::from_asset_vec(output.amount.clone());
            required_assets.merge(output_value);
        }

        for input in &self.core.mesh_tx_builder_body.inputs {
            match input {
                TxIn::PubKeyTxIn(pub_key_tx_in) => {
                    let input_value =
                        Value::from_asset_vec(pub_key_tx_in.tx_in.amount.clone().unwrap());
                    required_assets.negate_assets(input_value);
                }
                TxIn::SimpleScriptTxIn(simple_script_tx_in) => {
                    let input_value =
                        Value::from_asset_vec(simple_script_tx_in.tx_in.amount.clone().unwrap());
                    required_assets.negate_assets(input_value);
                }
                TxIn::ScriptTxIn(script_tx_in) => {
                    let input_value =
                        Value::from_asset_vec(script_tx_in.tx_in.amount.clone().unwrap());
                    required_assets.negate_assets(input_value);
                }
            }
        }

        for mint in &self.core.mesh_tx_builder_body.mints {
            let mint_amount = Asset::new(
                mint.policy_id.clone() + &mint.asset_name,
                mint.amount.to_string(),
            );
            required_assets.negate_asset(mint_amount);
        }

        let selected_inputs =
            select_utxos(extra_inputs, required_assets, threshold.to_string()).unwrap();

        for input in selected_inputs {
            self.core.mesh_csl.add_tx_in(PubKeyTxIn {
                tx_in: TxInParameter {
                    tx_hash: input.input.tx_hash.clone(),
                    tx_index: input.input.output_index,
                    amount: Some(input.output.amount.clone()),
                    address: Some(input.output.address.clone()),
                },
            })?;
            self.core
                .mesh_tx_builder_body
                .inputs
                .push(TxIn::PubKeyTxIn(PubKeyTxIn {
                    tx_in: TxInParameter {
                        tx_hash: input.input.tx_hash,
                        tx_index: input.input.output_index,
                        amount: Some(input.output.amount),
                        address: Some(input.output.address),
                    },
                }));
        }
        Ok(())
    }
}

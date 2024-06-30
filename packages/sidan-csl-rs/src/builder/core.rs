use cardano_serialization_lib::JsError;

use crate::{
    core::builder::{IMeshCSL, MeshCSL},
    csl,
    model::*,
    *,
};

use super::{interface::MeshTxBuilderCore, IMeshTxBuilderCore};

/// ## WASM Transaction building method
///
/// Serialize the transaction body
///
/// ### Arguments
///
/// * `mesh_tx_builder_body_json` - The transaction builder body information, serialized as JSON string
/// * `params_json` - Optional protocol parameters, default as Cardano mainnet configuration, serialized as JSON string
///
/// ### Returns
///
/// * `String` - the built transaction hex
#[wasm_bindgen]
pub fn js_serialize_tx_body(mesh_tx_builder_body_json: &str, params_json: &str) -> String {
    let mesh_tx_builder_body: MeshTxBuilderBody = serde_json::from_str(mesh_tx_builder_body_json)
        .expect("Error deserializing transaction body");

    let params: Option<Protocol> = match serde_json::from_str(params_json) {
        Ok(params) => Some(params),
        Err(_) => None,
    };

    serialize_tx_body(mesh_tx_builder_body, params).unwrap()
}

// #[test]
// fn test_js_serialize_tx_body() {
//     let mesh_tx_builder_body_json = r#"{"inputs":[{"pubKeyTxIn":{"type":"PubKey","txIn":{"txHash":"b789161dd0906ff55dbbbb2535d81e76568fcb0ebec24f5c554558dba83da0a8","txIndex":0,"amount":[{"unit":"lovelace","quantity":"1000000000"}],"address":"addr_test1qrfdlg620vjgsayzyljeyt2m5c0m6k9nhnhz2rupz3lt50x5wfge2g6j5q86l6qq553z5rapruml9ed5hmdgap4ldv7s58qmhq"}}}],"outputs":[{"address":"addr_test1qrfdlg620vjgsayzyljeyt2m5c0m6k9nhnhz2rupz3lt50x5wfge2g6j5q86l6qq553z5rapruml9ed5hmdgap4ldv7s58qmhq","amount":[{"unit":"95cee82d7791c57fc5aed517d12155f247655bf6b1908292b5a2e9cc","quantity":"1"}],"datum":null,"referenceScript":null}],"collaterals":[{"type":"PubKey","txIn":{"txHash":"b789161dd0906ff55dbbbb2535d81e76568fcb0ebec24f5c554558dba83da0a8","txIndex":0,"amount":[{"unit":"lovelace","quantity":"1000000000"}],"address":"addr_test1qrfdlg620vjgsayzyljeyt2m5c0m6k9nhnhz2rupz3lt50x5wfge2g6j5q86l6qq553z5rapruml9ed5hmdgap4ldv7s58qmhq"}}],"requiredSignatures":[],"referenceInputs":[],"mints":[{"type":"Plutus","policyId":"95cee82d7791c57fc5aed517d12155f247655bf6b1908292b5a2e9cc","assetName":"","amount":1,"redeemer":{"data":"d87980","exUnits":{"mem":7000000,"steps":3000000000}},"scriptSource":{"providedScriptSource":{"scriptCbor":"59022c5902290100003323232323232323232222325333007323232323232533300d3370e90000008991919299980819b87480000044c8c8c8c94ccc050cdc3a400000229445281809000991980080080111299980b8008a6103d87a80001323253330163375e026601e6028004266e9520003301a0024bd70099802002000980d801180c8009bac3016001300e00713232323300100100222533301700114a226464a66602ca66602c66e3cdd71807801004099b88375a60366038603800490000a5113300400400114a060360046eb0c064004c8c8cc004004008894ccc05c00452f5c0264666444646600200200644a66603a00220062646603e6e9ccc07cdd48031980f9ba9375c60380026603e6ea0dd6980e800a5eb80cc00c00cc084008c07c004dd7180b0009bab301700133003003301b0023019001323300100100222533301600114bd6f7b630099191919299980b99b8f48900002100313301b337606ea4008dd3000998030030019bab3018003375c602c004603400460300026eacc054c058c058c058c058c03801cc03802cdd7180980098058010a50300b00130100013010002300e00130060032300d00114984d958c94ccc01ccdc3a40000022a666014600a0062930b0a99980399b874800800454ccc028c01400c52616163005002230053754002460066ea80055cd2ab9d5573caae7d5d02ba157449812bd8799fd8799f5820b789161dd0906ff55dbbbb2535d81e76568fcb0ebec24f5c554558dba83da0a8ff00ff0001","languageVersion":"V2"}}}],"changeAddress":"addr_test1qrfdlg620vjgsayzyljeyt2m5c0m6k9nhnhz2rupz3lt50x5wfge2g6j5q86l6qq553z5rapruml9ed5hmdgap4ldv7s58qmhq","metadata":[],"validityRange":{"invalidBefore":null,"invalidHereafter":null},"certificates":[],"signingKey":["5820b7cbcc113d2fe1c6f97d858c2e512459b36034c67f630749567d8783757394c7"],"withdrawals":[]}"#;
//     let params_json = r#"{"epoch":0,"coinsPerUTxOSize":"4310","priceMem":0.0577,"priceStep":0.0000721,"minFeeA":44,"minFeeB":155381,"keyDeposit":"2000000","maxTxSize":16384,"maxValSize":"5000","poolDeposit":"500000000","maxCollateralInputs":3,"decentralisation":0,"maxBlockSize":98304,"collateralPercent":150,"maxBlockHeaderSize":1100,"minPoolCost":"340000000","maxTxExMem":"16000000","maxTxExSteps":"10000000000","maxBlockExMem":"80000000","maxBlockExSteps":"40000000000"}"#;
//     let tx_hex = js_serialize_tx_body(mesh_tx_builder_body_json, params_json);
//     println!("tx_hex: {:?}", tx_hex);
// }

/// ## Transaction building method
///
/// Serialize the transaction body
///
/// ### Arguments
///
/// * `mesh_tx_builder_body` - The transaction builder body information
/// * `params` - Optional protocol parameters, default as Cardano mainnet configuration
///
/// ### Returns
///
/// * `String` - the built transaction hex
pub fn serialize_tx_body(
    mesh_tx_builder_body: MeshTxBuilderBody,
    params: Option<Protocol>,
) -> Result<String, JsError> {
    let mut mesh_csl = MeshCSL::new(params);

    println!("1");
    MeshTxBuilderCore::add_all_inputs(&mut mesh_csl, mesh_tx_builder_body.inputs.clone())?;
    println!("2");
    MeshTxBuilderCore::add_all_outputs(&mut mesh_csl, mesh_tx_builder_body.outputs.clone())?;
    println!("3");
    MeshTxBuilderCore::add_all_collaterals(
        &mut mesh_csl,
        mesh_tx_builder_body.collaterals.clone(),
    )?;
    println!("4");
    MeshTxBuilderCore::add_all_reference_inputs(
        &mut mesh_csl,
        mesh_tx_builder_body.reference_inputs.clone(),
    )?;
    println!("5");
    MeshTxBuilderCore::add_all_withdrawals(
        &mut mesh_csl,
        mesh_tx_builder_body.withdrawals.clone(),
    )?;
    println!("6");
    MeshTxBuilderCore::add_all_mints(&mut mesh_csl, mesh_tx_builder_body.mints.clone())?;
    println!("7");
    MeshTxBuilderCore::add_all_certificates(
        &mut mesh_csl,
        mesh_tx_builder_body.certificates.clone(),
    )?;
    println!("8");
    MeshTxBuilderCore::add_validity_range(
        &mut mesh_csl,
        mesh_tx_builder_body.validity_range.clone(),
    );
    println!("9");
    MeshTxBuilderCore::add_all_required_signature(
        &mut mesh_csl,
        mesh_tx_builder_body.required_signatures.clone(),
    )?;
    println!("10");
    MeshTxBuilderCore::add_all_metadata(&mut mesh_csl, mesh_tx_builder_body.metadata.clone())?;
    println!("11");

    mesh_csl.add_script_hash()?;
    // if self.mesh_tx_builder_body.change_address != "" {
    //     let collateral_inputs = self.mesh_tx_builder_body.collaterals.clone();
    //     let collateral_vec: Vec<u64> = collateral_inputs
    //         .into_iter()
    //         .map(|pub_key_tx_in| {
    //             let assets = pub_key_tx_in.tx_in.amount.unwrap();
    //             let lovelace = assets
    //                 .into_iter()
    //                 .find(|asset| asset.unit == "lovelace")
    //                 .unwrap();
    //             lovelace.quantity.parse::<u64>().unwrap()
    //         })
    //         .collect();
    //     let total_collateral: u64 = collateral_vec.into_iter().sum();

    //     let collateral_estimate: u64 = (150
    //         * self
    //             .tx_builder
    //             .min_fee()
    //             .unwrap()
    //             .checked_add(&to_bignum(10000))
    //             .unwrap()
    //             .to_string()
    //             .parse::<u64>()
    //             .unwrap())
    //         / 100;

    //     let mut collateral_return_needed = false;
    // if (total_collateral - collateral_estimate) > 0 {
    // let collateral_estimate_output = csl::TransactionOutput::new(
    //     &csl::address::Address::from_bech32(&self.mesh_tx_builder_body.change_address)
    //         .unwrap(),
    //     &csl::utils::Value::new(&to_bignum(collateral_estimate)),
    // );

    // let min_ada = csl::utils::min_ada_for_output(
    //     &collateral_estimate_output,
    //     &csl::DataCost::new_coins_per_byte(&to_bignum(4310)),
    // )
    // .unwrap()
    // .to_string()
    // .parse::<u64>()
    // .unwrap();

    // if total_collateral - collateral_estimate > min_ada {
    //     self.tx_builder
    //         .set_collateral_return(&csl::TransactionOutput::new(
    //             &csl::address::Address::from_bech32(
    //                 &self.mesh_tx_builder_body.change_address,
    //             )
    //             .unwrap(),
    //             &csl::utils::Value::new(&to_bignum(total_collateral)),
    //         ));

    //     self.tx_builder
    //         .set_total_collateral(&to_bignum(total_collateral));

    //     collateral_return_needed = true;
    // }
    // }
    // self.add_change(self.mesh_tx_builder_body.change_address.clone());
    // if collateral_return_needed {
    //     self.add_collateral_return(self.mesh_tx_builder_body.change_address.clone());
    // }
    // }
    mesh_csl.add_change(
        mesh_tx_builder_body.change_address.clone(),
        mesh_tx_builder_body.change_datum.clone(),
    )?;
    mesh_csl.build_tx()
}

impl IMeshTxBuilderCore for MeshTxBuilderCore {
    fn new_core(params: Option<Protocol>) -> Self {
        Self {
            mesh_csl: MeshCSL::new(params),
            mesh_tx_builder_body: MeshTxBuilderBody {
                inputs: vec![],
                outputs: vec![],
                collaterals: vec![],
                required_signatures: JsVecString::new(),
                reference_inputs: vec![],
                withdrawals: vec![],
                mints: vec![],
                change_address: "".to_string(),
                change_datum: None,
                certificates: vec![],
                metadata: vec![],
                validity_range: ValidityRange {
                    invalid_before: None,
                    invalid_hereafter: None,
                },
                signing_key: JsVecString::new(),
            },
            tx_evaluation_multiplier_percentage: 110,
        }
    }

    fn complete_signing(&mut self) -> String {
        let signing_keys = self.mesh_tx_builder_body.signing_key.clone();
        self.add_all_signing_keys(signing_keys);
        self.mesh_csl.tx_hex.to_string()
    }

    fn add_all_signing_keys(&mut self, signing_keys: JsVecString) {
        if !signing_keys.is_empty() {
            self.mesh_csl.add_signing_keys(signing_keys);
        }
    }

    fn add_all_inputs(mesh_csl: &mut MeshCSL, inputs: Vec<TxIn>) -> Result<(), JsError> {
        for input in inputs {
            match input {
                TxIn::PubKeyTxIn(pub_key_tx_in) => mesh_csl.add_tx_in(pub_key_tx_in)?,
                TxIn::SimpleScriptTxIn(simple_script_tx_in) => {
                    mesh_csl.add_simple_script_tx_in(simple_script_tx_in)?
                }
                TxIn::ScriptTxIn(script_tx_in) => mesh_csl.add_script_tx_in(script_tx_in)?,
            };
        }
        mesh_csl.tx_builder.set_inputs(&mesh_csl.tx_inputs_builder);
        Ok(())
    }

    fn add_all_outputs(mesh_csl: &mut MeshCSL, outputs: Vec<Output>) -> Result<(), JsError> {
        for output in outputs {
            mesh_csl.add_output(output)?;
        }
        Ok(())
    }

    fn add_all_collaterals(
        mesh_csl: &mut MeshCSL,
        collaterals: Vec<PubKeyTxIn>,
    ) -> Result<(), JsError> {
        let mut collateral_builder = csl::TxInputsBuilder::new();
        for collateral in collaterals {
            mesh_csl.add_collateral(&mut collateral_builder, collateral)?
        }
        mesh_csl.tx_builder.set_collateral(&collateral_builder);
        Ok(())
    }

    fn add_all_reference_inputs(
        mesh_csl: &mut MeshCSL,
        ref_inputs: Vec<RefTxIn>,
    ) -> Result<(), JsError> {
        for ref_input in ref_inputs {
            mesh_csl.add_reference_input(ref_input)?;
        }
        Ok(())
    }

    fn add_all_withdrawals(
        mesh_csl: &mut MeshCSL,
        withdrawals: Vec<Withdrawal>,
    ) -> Result<(), JsError> {
        for withdrawal in withdrawals {
            match withdrawal {
                Withdrawal::PubKeyWithdrawal(pub_key_withdrawal) => {
                    mesh_csl.add_pub_key_withdrawal(pub_key_withdrawal)?
                }
                Withdrawal::PlutusScriptWithdrawal(plutus_script_withdrawal) => {
                    mesh_csl.add_plutus_withdrawal(plutus_script_withdrawal)?
                }
            }
        }
        mesh_csl
            .tx_builder
            .set_withdrawals_builder(&mesh_csl.tx_withdrawals_builder);
        Ok(())
    }

    fn add_all_mints(mesh_csl: &mut MeshCSL, mints: Vec<MintItem>) -> Result<(), JsError> {
        let mut mint_builder = csl::MintBuilder::new();
        println!("6-1");
        for (index, mint) in mints.into_iter().enumerate() {
            println!("6-2");
            match mint.type_ {
                MintItemType::Plutus => {
                    mesh_csl.add_plutus_mint(&mut mint_builder, mint, index as u64)?
                }
                MintItemType::Native => mesh_csl.add_native_mint(&mut mint_builder, mint)?,
            };
        }
        println!("6-3");
        mesh_csl.tx_builder.set_mint_builder(&mint_builder);
        println!("6-4");
        Ok(())
    }

    fn add_all_certificates(
        mesh_csl: &mut MeshCSL,
        certificates: Vec<Certificate>,
    ) -> Result<(), JsError> {
        let mut certificates_builder = csl::CertificatesBuilder::new();
        for cert in certificates {
            match cert {
                Certificate::RegisterPool(register_pool) => {
                    mesh_csl.add_register_pool_cert(&mut certificates_builder, register_pool)?
                }
                Certificate::RegisterStake(register_stake) => {
                    mesh_csl.add_register_stake_cert(&mut certificates_builder, register_stake)?
                }
                Certificate::DelegateStake(delegate_stake) => {
                    mesh_csl.add_delegate_stake_cert(&mut certificates_builder, delegate_stake)?
                }
                Certificate::DeregisterStake(deregister_stake) => mesh_csl
                    .add_deregister_stake_cert(&mut certificates_builder, deregister_stake)?,
                Certificate::RetirePool(retire_pool) => {
                    mesh_csl.add_retire_pool_cert(&mut certificates_builder, retire_pool)?
                }
                Certificate::VoteDelegation(vote_delegation) => {
                    mesh_csl.add_vote_delegation_cert(&mut certificates_builder, vote_delegation)?
                }
                Certificate::StakeAndVoteDelegation(stake_and_vote_delegation) => mesh_csl
                    .add_stake_and_vote_delegation_cert(
                        &mut certificates_builder,
                        stake_and_vote_delegation,
                    )?,
                Certificate::StakeRegistrationAndDelegation(stake_registration_and_delegation) => {
                    mesh_csl.add_stake_registration_and_delegation_cert(
                        &mut certificates_builder,
                        stake_registration_and_delegation,
                    )?
                }
                Certificate::VoteRegistrationAndDelegation(vote_registration_and_delegation) => {
                    mesh_csl.add_vote_registration_and_delgation_cert(
                        &mut certificates_builder,
                        vote_registration_and_delegation,
                    )?
                }
                Certificate::StakeVoteRegistrationAndDelegation(
                    stake_vote_registration_and_delegation,
                ) => mesh_csl.add_stake_vote_registration_and_delegation_cert(
                    &mut certificates_builder,
                    stake_vote_registration_and_delegation,
                )?,
                Certificate::CommitteeHotAuth(committee_hot_auth) => mesh_csl
                    .add_committee_hot_auth_cert(&mut certificates_builder, committee_hot_auth)?,
                Certificate::CommitteeColdResign(commitee_cold_resign) => mesh_csl
                    .add_commitee_cold_resign_cert(
                        &mut certificates_builder,
                        commitee_cold_resign,
                    )?,
                Certificate::DRepRegistration(drep_registration) => mesh_csl
                    .add_drep_registration_cert(&mut certificates_builder, drep_registration)?,
                Certificate::DRepDeregistration(drep_deregistration) => mesh_csl
                    .add_drep_deregistration_cert(&mut certificates_builder, drep_deregistration)?,
                Certificate::DRepUpdate(drep_update) => {
                    mesh_csl.add_drep_update_cert(&mut certificates_builder, drep_update)?
                }
            }
        }
        mesh_csl.tx_builder.set_certs_builder(&certificates_builder);
        Ok(())
    }

    fn add_validity_range(mesh_csl: &mut MeshCSL, validity_range: ValidityRange) {
        if validity_range.invalid_before.is_some() {
            mesh_csl.add_invalid_before(validity_range.invalid_before.unwrap())
        }
        if validity_range.invalid_hereafter.is_some() {
            mesh_csl.add_invalid_hereafter(validity_range.invalid_hereafter.unwrap())
        }
    }

    fn add_all_required_signature(
        mesh_csl: &mut MeshCSL,
        required_signatures: JsVecString,
    ) -> Result<(), JsError> {
        for pub_key_hash in required_signatures {
            mesh_csl.add_required_signature(pub_key_hash)?;
        }
        Ok(())
    }

    fn add_all_metadata(
        mesh_csl: &mut MeshCSL,
        all_metadata: Vec<Metadata>,
    ) -> Result<(), JsError> {
        for metadata in all_metadata {
            mesh_csl.add_metadata(metadata)?;
        }
        Ok(())
    }

    // fn add_collateral_return(&mut self, change_address: String) {
    //     let current_fee = self
    //         .tx_builder
    //         .get_fee_if_set()
    //         .unwrap()
    //         .to_string()
    //         .parse::<u64>()
    //         .unwrap();

    //     let collateral_amount = 150 * ((current_fee / 100) + 1);
    //     let _ = self
    //         .tx_builder
    //         .set_total_collateral_and_return(
    //             &to_bignum(collateral_amount),
    //             &csl::address::Address::from_bech32(&change_address).unwrap(),
    //         )
    //         .unwrap();
    // }
}

impl Default for MeshTxBuilderCore {
    fn default() -> Self {
        Self::new_core(None)
    }
}

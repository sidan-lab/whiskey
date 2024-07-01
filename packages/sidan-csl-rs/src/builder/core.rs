use builder::TxBuildResult;
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
pub fn js_serialize_tx_body(mesh_tx_builder_body_json: &str, params_json: &str) -> TxBuildResult {
    let mesh_tx_builder_body: MeshTxBuilderBody = serde_json::from_str(mesh_tx_builder_body_json)
        .expect("Error deserializing transaction body");

    let params: Option<Protocol> = match serde_json::from_str(params_json) {
        Ok(params) => Some(params),
        Err(_) => None,
    };

    match serialize_tx_body(mesh_tx_builder_body, params) {
        Ok(tx_hex) => TxBuildResult::new("success".to_string(), tx_hex.to_string()),
        Err(e) => TxBuildResult::new("failure".to_string(), format!("{:?}", e)),
    }
}

#[test]
fn test_js_serialize_tx_body() {
    let mesh_tx_builder_body_json = r#"{"inputs":[{"scriptTxIn":{"txIn":{"txHash":"eee69f04f6f23646a55be754c36e1726134b35022e035983f813434beca61bca","txIndex":0,"amount":[{"unit":"lovelace","policy_id":null,"asset_name":"lovelace","quantity":"2361880"},{"unit":"db0bfb086bee3d818a34c23bd85d80151aec089a4c6c49f88cc209e5","policy_id":"db0bfb086bee3d818a34c23bd85d80151aec089a4c6c49f88cc209e5","asset_name":"","quantity":"1"}],"address":null},"scriptTxIn":{"scriptSource":{"providedScriptSource":{"scriptCbor":"59073a590737010000323232323232323232222323232323232533300c3232323232323232323232323232323232533301d3370e9001000899191919191919191919191919191919191919191919191919191919299981c99b87480000044c8c8c8c8cc004004008894ccc1040045280991929998202999820299982019b8f375c6058004024266e3cdd718168010038a5013370e6eb4c0bc009200214a029444cc010010004c114008dd618218009812800981180b9981399813a450644617461202800330290083302c00848009221012900153330393370e9001000899191919191929998211822801099299982019b8748010c0fc0044c8c8c8c94ccc1114ccc110c8cc004004040894ccc124004528099192999824198198100010a51133004004001304d002375c609600226605e03801a2940400452819baf0023232323232323374a90001982698270031982698270029982698270021982698270019982698270011982698270009982698271827800998269ba700f3304d375201a97ae0304e001304d001304c001304b001304a001304104030390013046001303e00116302e303d001163043001323300100101e22533304200114bd70099192999820a99982099baf302d303f00201513370e64646464a66608a66e1d20020011480004dd698251821801182180099299982219b8748008004530103d87a8000132323300100100222533304a00114c103d87a8000132323232533304b3371e9110000213374a9000198279ba80014bd700998030030019bad304c003375c6094004609c00460980026eacc124c108008c108004c8cc004004008894ccc11c0045300103d87a800013232323253330483371e03c004266e9520003304c374c00297ae0133006006003375660920066eb8c11c008c12c008c124004dd59817181f8012400429404cc114008cc0100100044cc010010004c118008c110004dd7182080098208011bac303f00130370351533303933024011002132533303d00114a0264a66607c002264646464a66607ea66607e66e3c0040544cdc78012450014a0266e1c00d200114a06eb8c10c00cdd7182118218011bad304130423042001375860800042940c100004c08cc088058528181b81a1bae303c001303c002375860740026074002607200260700046eb4c0d8004c0d8004c0d4008dd718198009819801181880098188011bae302f00130270263758605a002605a0026058002605600260540046eacc0a0004c0a0004c09c008dd6181280098128009812000980d8020a50301b00130200013020002301e00130160132323300100100222533301d00114bd6f7b630099191919299980f19b8f4881000021003133022337606ea4008dd3000998030030019bab301f003375c603a0046042004603e0024646600200200444a666038002297ae01323332223233001001003225333022001100313233024374e660486ea4018cc090dd49bae30210013302437506eb4c0880052f5c066006006604c00460480026eb8c06c004dd5980e000998018019810001180f00091191980080080191299980e0008a5013232533301b3371e00400a29444cc010010004c080008dd7180f0009180d0009180c980d0009119b8a0020012301730183018001300100122253330123370e002900109980224410033700004903009980219980180199b8600200133706002900a19b803370600400290301119b8b0010023001001222533300f33710004900a0800899980180199b8300248050cdc1000a402829309b2b19299980619b874800000454ccc03cc028020526161533300c3370e90010008991919192999809980b00109924c6601000646eb800458dd7180a000980a0011bac3012001300a0081533300c3370e90020008a99980798050040a4c2c2c601400e600200e464a66601666e1d2000001132323232323232323232323232323232323253330203023002132323232498cc0600188dd7000980c003980b806180b0078b1bae302100130210023758603e002603e004603a002603a0046eb8c06c004c06c008dd6980c800980c801180b800980b8011bae3015001301500230130013013002375c602200260120042c601200244646600200200644a66601e00229309919801801980980118019808800919299980499b87480000044c8c8c8c94ccc040c04c0084c8c92632533300f3370e900000089919299980a180b80109924c64a66602466e1d2000001132325333017301a002132498c03800458c060004c04000854ccc048cdc3a40040022646464646464a666036603c0042930b1bad301c001301c002375a603400260340046eb4c060004c04000858c04000458c054004c03400c54ccc03ccdc3a40040022a666024601a0062930b0b180680118038018b18088009808801180780098038010b1803800919299980419b87480000044c8c94ccc034c04000852616375c601c002600c0042a66601066e1d200200113232533300d3010002149858dd7180700098030010b1803000918029baa001230033754002ae6955ceaab9e5573eae815d0aba21","languageVersion":"v2"}},"datumSource":{"inlineDatumSource":{"txHash":"eee69f04f6f23646a55be754c36e1726134b35022e035983f813434beca61bca","txIndex":0}},"redeemer":{"data":"d87980","exUnits":{"mem":7000000,"steps":3000000000}}}}}],"outputs":[{"address":"addr_test1wzug5lq0d3sxp27v7mxnlg2er783q6v6uwkweq5xjtseyfqdf3smk","amount":[{"unit":"db0bfb086bee3d818a34c23bd85d80151aec089a4c6c49f88cc209e5","quantity":"1"}],"datum":{"inline":"d8799f581cdb0bfb086bee3d818a34c23bd85d80151aec089a4c6c49f88cc209e5d8799fd87a9f581cb88a7c0f6c6060abccf6cd3fa1591f8f10699ae3acec828692e19224ffd87a80ff581c023be3a796f03fe37f304281d9b8e15f41998c18dc1e09d17461a9a1d8799fd87a9f581c585b2ca7f4272df1fe600d2b720fb6719c0b83275584490369a02117ffd87a80ff01581c157bdf30353e0f57c0eb16f1aafb652c78a58ce9a2b6077f79ad8c82d8799fd87a9f581cd88dbb5b7e2b40063d4a59cacd49564bf867f8795a8c331840c3207fffd87a80ff9f581c5867c3b8e27840f556ac268b781578b14c5661fc63ee720dbeab663fff581cbbb1a36cc3e076d689176e77374ca26d4e09047c9d9dbd10ab0dcdaeff"},"referenceScript":null}],"collaterals":[{"txIn":{"txHash":"eee69f04f6f23646a55be754c36e1726134b35022e035983f813434beca61bca","txIndex":1,"amount":[{"unit":"lovelace","quantity":"996635438"}],"address":"addr_test1qpvx0sacufuypa2k4sngk7q40zc5c4npl337uusdh64kv0uafhxhu32dys6pvn6wlw8dav6cmp4pmtv7cc3yel9uu0nq93swx9"}}],"requiredSignatures":[],"referenceInputs":[],"mints":[],"changeAddress":"addr_test1qpvx0sacufuypa2k4sngk7q40zc5c4npl337uusdh64kv0uafhxhu32dys6pvn6wlw8dav6cmp4pmtv7cc3yel9uu0nq93swx9","metadata":[],"validityRange":{"invalidBefore":null,"invalidHereafter":null},"certificates":[],"signingKey":[],"withdrawals":[]}"#;
    let params_json = r#"{"epoch":0,"coinsPerUTxOSize":"4310","priceMem":0.0577,"priceStep":0.0000721,"minFeeA":44,"minFeeB":155381,"keyDeposit":"2000000","maxTxSize":16384,"maxValSize":"5000","poolDeposit":"500000000","maxCollateralInputs":3,"decentralisation":0,"maxBlockSize":98304,"collateralPercent":150,"maxBlockHeaderSize":1100,"minPoolCost":"340000000","maxTxExMem":"16000000","maxTxExSteps":"10000000000","maxBlockExMem":"80000000","maxBlockExSteps":"40000000000"}"#;
    let tx_build_result = js_serialize_tx_body(mesh_tx_builder_body_json, params_json);
    println!("{:?}", tx_build_result);
}

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

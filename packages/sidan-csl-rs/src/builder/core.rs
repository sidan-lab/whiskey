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
    let mesh_tx_builder_body: MeshTxBuilderBody =
        match serde_json::from_str(mesh_tx_builder_body_json) {
            Ok(mesh_tx_builder_body) => mesh_tx_builder_body,
            Err(e) => {
                return TxBuildResult::new("failure".to_string(), format!("Invalid JSON: {:?}", e))
            }
        };

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
    let mesh_tx_builder_body_json = r#"{"inputs":[{"pubKeyTxIn":{"txIn":{"txHash":"345bf96842f9ff9ee1b5a3bcd704f728cb6ebc90907f72e01e71fd87a4cdcd9f","txIndex":0,"amount":[{"unit":"lovelace","quantity":"20000000"}],"address":"addr_test1qpvx0sacufuypa2k4sngk7q40zc5c4npl337uusdh64kv0uafhxhu32dys6pvn6wlw8dav6cmp4pmtv7cc3yel9uu0nq93swx9"}}}],"outputs":[{"address":"addr_test1qzamrgmvc0s8d45fzah8wd6v5fk5uzgy0jwem0gs4vxumt3qr062jqd50h9upf9k3h5tkw4vx5ww7fjcy9yd5swrld4slfpasu","amount":[],"datum":null,"referenceScript":{"code":"590ae5590ae20100003323232323232323232232223232533300932323232323232323232323232323232323232323232323232323232323232323232533302b3370e90000008991919191919299981899b8748000c0c00084c8c8c8c8c8c8c8c8c8c8c8c8c8c94ccc10804403c4c94ccc10c0044c8c94ccc108cdc3a40086082002264646464646464646464646464646464646464646464646464a6660bc004002264a6660be002264a6660c000a006264a6660c2002264a6660c401000a264a6660c6002264a6660c801600e264a6660ca00226464a6660c866e1d20043063001132323253330673370e900218330008991919299983519b8748010c1a40044c8c8c94ccc1b4cdc3a400860d800226464646464646464646464646464a6660f6a6660f6a6660f6a6660f6a6660f6a6660f6a6660f6010200e2940401852808028a50100414a020062940400852808008a503371260ba6eacc1a8c1e006120043371260b86eacc1a4c1dc06520043371260b66eacc1a0c1d806920043371260b46eacc19cc1d406d2004533307653330765333076533307653330763375e60be60e80106e9c0e84cdc39bad307b307c307c307c307400848000528099b8f375c60ba60e801007029404cdc39bad307b307c307c307c307c3074008480005280a99983b19b8748008c1dcdd5183d983e183e183e183e183e183a0040a5113330764a2941288a50132323300100102522533307c00114a026464a6660f666e3c0080145288998020020009840008011bae307e001375c60cc60e80102940cdd780519ba548000cc1e4dd381d9983c9ba903d33079375206e660f2981010100330794c1010000330793330754a2980103d87a80004c0103d87980004bd7019baf00c3374a90001983c1ba904233078374e080660f06e9c0f8cc1e0dd481e25eb80cdd78071919191919ba548000cc1ecc1f0010cc1ecc1f000ccc1ecc1f0008cc1ecc1f0004cc1ecdd419b8002a48008c1f4c1f4004c1f0004c1ec004c1e8004c1c40c0c158010c154018c94ccc1c4cdc3a400000226464646464646464a6660f860fe00426464931983980211bae001330720052375c0022c6eb8c1f4004c1f4008dd6183d800983d8011bac30790013079002375c60ee00260de0122c60de01060aa01460e600260d60022c60a660d401460e000260d00022c60a060ce01260da00260ca0022c609a60c801060d400260c40022c609460c200e60ce01801060ce01660ca01200c60ca01060c600c00860c600a60c200600460c20042940dd6182f0021bac305d305e003375860b860ba60ba0046eb0c16cc170c170c170004cc164dd3999981d017807028a450033059374e666607405e01401866082660829110644617461202800304200848901290033059374e666607405e00800c002660b26e9ccccc0e80bc010018cc104cc1040052201012d003042480012f5c06608091102502d003041007375860b000260b000460ac00260ac0046eb8c150004c150008dd698290009829001182800098280011bae304e001304e002304c001304c0013043002302a00130480013040001163028303f3031303f00130450120103045011375c608600260860046eb0c104004c104008dd6181f800981f8011bae303d001303d0023758607600260760046eb0c0e4004c0e4008dd7181b80098178010008a99981819b8748008c0bc0044c8c8cc004004008894ccc0d800452889919299981aa99981a99b8f375c603c004012266e20dd6980e0012400029444cc010010004528181d0011bac30380013018323300100100822533303500114bd6f7b630099191919299981b19b8f488100002100313303a337606ea4008dd3000998030030019bab3037003375c606a0046072004606e0022940c0c0008dd6181800099bb0028374e66036014466e1cccc074dd5980f1816180f181600081524410048008dd7181880098148050a5030290093756605c002605c002605a0046eb0c0ac004c0ac004c0a8008dd618140009810001981300098130011812000980e00c118041803000919299980f19b87480000044c8c8c8c8c8c8c8c8c8c8c8c94ccc0b4c0c00084c9263302300b2375c0022c66e1d2002302a3754605c002605c0046eb4c0b0004c0b0008dd6981500098150011bae30280013028002375c604c002604c0046eb0c090004c07000858c0700048888cc03401094ccc080cdd79804980f000802099b8733300f37566020603c00200600490010a50232533301c3370e900000089919191919191919191919191919191919192999818981a00109919191924c6605400c46eb8004c08401cc080030c07c03c58dd7181900098190011bac30300013030002302e001302e002375c605800260580046eb4c0a8004c0a8008c0a0004c0a0008dd718130009813001181200098120011bae3022001301a00216301a0012301f302030200012323300100100222533301e00114bd7009919991119198008008019129998120008801899198131ba733026375200c6604c6ea4dd71811800998131ba8375a604800297ae03300300330280023026001375c603a0026eacc078004cc00c00cc088008c0800048c074004c004004894ccc06800452000133700900119801001180e8009119b8a002001233006001330090014800888c8cc00400400c894ccc06400452f5c026464a666030600a00426603800466008008002266008008002603a0046036002444646464a66602e66e1d20020011480004dd6980e180a801180a80099299980b19b8748008004530103d87a8000132323300100100222533301c00114c103d87a8000132323232533301d3371e014004266e95200033021375000297ae0133006006003375a603c0066eb8c070008c080008c078004dd5980d980a001180a000991980080080211299980c8008a6103d87a8000132323232533301a3371e010004266e9520003301e374c00297ae0133006006003375660360066eb8c064008c074008c06c0048c058c05c0048c94ccc044cdc3a4000002264646464a6660306036004264649319299980b99b87480000044c8c94ccc070c07c0084c92632533301a3370e900000089919299980f981100109924c60260022c604000260300042a66603466e1d2002001132323232323253330233026002149858dd6981200098120011bad30220013022002375a604000260300042c60300022c603a002602a0062a66602e66e1d20020011533301a301500314985858c054008c03000c58c064004c064008c05c004c03c00858c03c004c0040048894ccc040cdc3800a40042660089110033700004903009980219980180199b8600200133706002900a19b803370600400290301119b8b0010023001001222533300d33710004900a0800899980180199b8300248050cdc1000a4028464a66601666e1d20000011323253330103013002149858dd7180880098048010a99980599b87480080044c8c94ccc040c04c00852616375c602200260120042c601200229309b2b19299980499b87480000044c8c8c8c8c8c8c8c8c8c8c8c8c8c94ccc068c0740084c8c8c8c926330130062375c0026602400e46eb8004cc0440288dd70009980800591bae00116375c603600260360046eb0c064004c064008dd6180b800980b8011bae301500130150023758602600260260046eb0c044004c044008dd7180780098038020a99980499b874800800454ccc030c01c0105261616300700322323300100100322533300e00114984c8cc00c00cc048008c00cc040004dd7000918029baa001230033754002ae6955ceaab9e5573eae815d0aba24c11e581cdb0bfb086bee3d818a34c23bd85d80151aec089a4c6c49f88cc209e50001","version":"V2"}}],"collaterals":[],"requiredSignatures":[],"referenceInputs":[],"mints":[],"changeAddress":"addr_test1qpvx0sacufuypa2k4sngk7q40zc5c4npl337uusdh64kv0uafhxhu32dys6pvn6wlw8dav6cmp4pmtv7cc3yel9uu0nq93swx9","metadata":[],"validityRange":{"invalidBefore":null,"invalidHereafter":null},"certificates":[],"signingKey":[],"withdrawals":[]}"#;
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

use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{csl, model::*};
use cardano_serialization_lib::{Credential, JsError};

use super::utils::{
    build_tx_builder, sign_transaction, to_bignum, to_csl_anchor, to_csl_drep, to_value,
};

pub trait IMeshCSL {
    fn new(params: Option<Protocol>) -> Self;
    fn add_tx_in(&mut self, input: PubKeyTxIn) -> Result<(), JsError>;
    fn add_simple_script_tx_in(&mut self, input: SimpleScriptTxIn) -> Result<(), JsError>;
    fn add_script_tx_in(&mut self, input: ScriptTxIn) -> Result<(), JsError>;
    fn add_output(&mut self, output: Output) -> Result<(), JsError>;
    fn add_collateral(
        &mut self,
        collateral_builder: &mut csl::TxInputsBuilder,
        collateral: PubKeyTxIn,
    ) -> Result<(), JsError>;
    fn add_reference_input(&mut self, ref_input: RefTxIn) -> Result<(), JsError>;
    fn add_pub_key_withdrawal(&mut self, withdrawal: PubKeyWithdrawal) -> Result<(), JsError>;
    fn add_plutus_withdrawal(&mut self, withdrawal: PlutusScriptWithdrawal) -> Result<(), JsError>;
    fn add_plutus_mint(
        &mut self,
        mint_builder: &mut csl::MintBuilder,
        mint: MintItem,
        index: u64,
    ) -> Result<(), JsError>;
    fn add_native_mint(
        &mut self,
        mint_builder: &mut csl::MintBuilder,
        mint: MintItem,
    ) -> Result<(), JsError>;
    fn add_register_pool_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        register_pool: RegisterPool,
    ) -> Result<(), JsError>;
    fn add_register_stake_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        register_stake: RegisterStake,
    ) -> Result<(), JsError>;
    fn add_delegate_stake_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        delegate_stake: DelegateStake,
    ) -> Result<(), JsError>;
    fn add_deregister_stake_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        deregister_stake: DeregisterStake,
    ) -> Result<(), JsError>;
    fn add_retire_pool_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        retire_pool: RetirePool,
    ) -> Result<(), JsError>;
    fn add_vote_delegation_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        vote_delegation: VoteDelegation,
    ) -> Result<(), JsError>;
    fn add_stake_and_vote_delegation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        stake_and_vote_delegation: StakeAndVoteDelegation,
    ) -> Result<(), JsError>;
    fn add_stake_registration_and_delegation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        stake_registration_and_delegation: StakeRegistrationAndDelegation,
    ) -> Result<(), JsError>;
    fn add_vote_registration_and_delgation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        vote_registration_and_delgation: VoteRegistrationAndDelegation,
    ) -> Result<(), JsError>;
    fn add_stake_vote_registration_and_delegation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        stake_vote_registration_and_delegation: StakeVoteRegistrationAndDelegation,
    ) -> Result<(), JsError>;
    fn add_committee_hot_auth_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        committee_hot_auth: CommitteeHotAuth,
    ) -> Result<(), JsError>;
    fn add_commitee_cold_resign_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        committee_cold_resign: CommitteeColdResign,
    ) -> Result<(), JsError>;
    fn add_drep_registration_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        drep_registration: DRepRegistration,
    ) -> Result<(), JsError>;
    fn add_drep_deregistration_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        drep_deregistration: DRepDeregistration,
    ) -> Result<(), JsError>;
    fn add_drep_update_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        drep_update: DRepUpdate,
    ) -> Result<(), JsError>;
    fn add_invalid_before(&mut self, invalid_before: u64);
    fn add_invalid_hereafter(&mut self, invalid_hereafter: u64);
    fn add_change(
        &mut self,
        change_address: String,
        change_datum: Option<Datum>,
    ) -> Result<(), JsError>;
    fn add_signing_keys(&mut self, signing_keys: JsVecString);
    fn add_required_signature(&mut self, pub_key_hash: String) -> Result<(), JsError>;
    fn add_metadata(&mut self, metadata: Metadata) -> Result<(), JsError>;
    fn add_script_hash(&mut self) -> Result<(), JsError>;
    fn build_tx(&mut self) -> Result<String, JsError>;
}

pub struct MeshCSL {
    pub tx_hex: String,
    pub tx_builder: csl::TransactionBuilder,
    pub tx_inputs_builder: csl::TxInputsBuilder,
    pub tx_withdrawals_builder: csl::WithdrawalsBuilder,
}

impl IMeshCSL for MeshCSL {
    fn new(params: Option<Protocol>) -> MeshCSL {
        MeshCSL {
            tx_hex: String::new(),
            tx_builder: build_tx_builder(params),
            tx_inputs_builder: csl::TxInputsBuilder::new(),
            tx_withdrawals_builder: csl::WithdrawalsBuilder::new(),
        }
    }

    fn add_tx_in(&mut self, input: PubKeyTxIn) -> Result<(), JsError> {
        self.tx_inputs_builder.add_regular_input(
            &csl::Address::from_bech32(&input.tx_in.address.unwrap())?,
            &csl::TransactionInput::new(
                &csl::TransactionHash::from_hex(&input.tx_in.tx_hash)?,
                input.tx_in.tx_index,
            ),
            &to_value(&input.tx_in.amount.unwrap()),
        )?;
        Ok(())
    }

    fn add_simple_script_tx_in(&mut self, input: SimpleScriptTxIn) -> Result<(), JsError> {
        match input.simple_script_tx_in {
            SimpleScriptTxInParameter::ProvidedSimpleScriptSource(script) => {
                self.tx_inputs_builder.add_native_script_input(
                    &csl::NativeScriptSource::new(&csl::NativeScript::from_hex(
                        &script.script_cbor,
                    )?),
                    &csl::TransactionInput::new(
                        &csl::TransactionHash::from_hex(&input.tx_in.tx_hash)?,
                        input.tx_in.tx_index,
                    ),
                    &to_value(&input.tx_in.amount.unwrap()),
                );
                Ok(())
            }
            SimpleScriptTxInParameter::InlineSimpleScriptSource(script) => {
                self.tx_inputs_builder.add_native_script_input(
                    &csl::NativeScriptSource::new_ref_input(
                        &csl::ScriptHash::from_hex(&script.simple_script_hash)?,
                        &csl::TransactionInput::new(
                            &csl::TransactionHash::from_hex(&script.ref_tx_in.tx_hash)?,
                            script.ref_tx_in.tx_index,
                        ),
                    ),
                    &csl::TransactionInput::new(
                        &csl::TransactionHash::from_hex(&input.tx_in.tx_hash)?,
                        input.tx_in.tx_index,
                    ),
                    &to_value(&input.tx_in.amount.unwrap()),
                );
                Ok(())
            } // Err(JsError::from_str(
              //     "Reference Native scripts not implemented",
              // )),
        }
    }

    fn add_script_tx_in(&mut self, input: ScriptTxIn) -> Result<(), JsError> {
        let datum_source = input.script_tx_in.datum_source.unwrap();
        let script_source = input.script_tx_in.script_source.unwrap();
        let redeemer = input.script_tx_in.redeemer.unwrap();
        let csl_datum: csl::DatumSource = match datum_source {
            DatumSource::ProvidedDatumSource(datum) => {
                csl::DatumSource::new(&csl::PlutusData::from_hex(&datum.data)?)
            }
            DatumSource::InlineDatumSource(datum) => {
                let ref_input = csl::TransactionInput::new(
                    &csl::TransactionHash::from_hex(&datum.tx_hash)?,
                    datum.tx_index,
                );
                csl::DatumSource::new_ref_input(&ref_input)
            }
        };

        let csl_script: csl::PlutusScriptSource = match script_source {
            ScriptSource::ProvidedScriptSource(script) => {
                let language_version: csl::Language = match script.language_version {
                    LanguageVersion::V1 => csl::Language::new_plutus_v1(),
                    LanguageVersion::V2 => csl::Language::new_plutus_v2(),
                    LanguageVersion::V3 => csl::Language::new_plutus_v3(),
                };
                csl::PlutusScriptSource::new(&csl::PlutusScript::from_hex_with_version(
                    &script.script_cbor,
                    &language_version,
                )?)
            }
            ScriptSource::InlineScriptSource(script) => {
                let language_version: csl::Language = match script.language_version {
                    LanguageVersion::V1 => csl::Language::new_plutus_v1(),
                    LanguageVersion::V2 => csl::Language::new_plutus_v2(),
                    LanguageVersion::V3 => csl::Language::new_plutus_v3(),
                };
                csl::PlutusScriptSource::new_ref_input(
                    &csl::ScriptHash::from_hex(&script.spending_script_hash)?,
                    &csl::TransactionInput::new(
                        &csl::TransactionHash::from_hex(&script.tx_hash)?,
                        script.tx_index,
                    ),
                    &language_version,
                    script.script_size,
                )
            }
        };

        let csl_redeemer: csl::Redeemer = csl::Redeemer::new(
            &csl::RedeemerTag::new_spend(),
            &to_bignum(0),
            &csl::PlutusData::from_hex(&redeemer.data)?,
            &csl::ExUnits::new(
                &to_bignum(redeemer.ex_units.mem),
                &to_bignum(redeemer.ex_units.steps),
            ),
        );
        self.tx_inputs_builder.add_plutus_script_input(
            &csl::PlutusWitness::new_with_ref(&csl_script, &csl_datum, &csl_redeemer),
            &csl::TransactionInput::new(
                &csl::TransactionHash::from_hex(&input.tx_in.tx_hash)?,
                input.tx_in.tx_index,
            ),
            &to_value(&input.tx_in.amount.unwrap()),
        );
        Ok(())
    }

    fn add_output(&mut self, output: Output) -> Result<(), JsError> {
        let mut output_builder = csl::TransactionOutputBuilder::new()
            .with_address(&csl::Address::from_bech32(&output.address)?);
        if output.datum.is_some() {
            let datum = output.datum.unwrap();

            match datum {
                Datum::Hash(data) => {
                    output_builder = output_builder
                        .with_data_hash(&csl::hash_plutus_data(&csl::PlutusData::from_hex(&data)?));
                }
                Datum::Inline(data) => {
                    output_builder =
                        output_builder.with_plutus_data(&csl::PlutusData::from_hex(&data)?);
                }
            };
        }

        if output.reference_script.is_some() {
            let output_script = output.reference_script.unwrap();
            let language_version: csl::Language = match output_script.language_version {
                LanguageVersion::V1 => csl::Language::new_plutus_v1(),
                LanguageVersion::V2 => csl::Language::new_plutus_v2(),
                LanguageVersion::V3 => csl::Language::new_plutus_v3(),
            };
            output_builder = output_builder.with_script_ref(&csl::ScriptRef::new_plutus_script(
                &csl::PlutusScript::from_hex_with_version(
                    &output_script.script_cbor,
                    &language_version,
                )?,
            ))
        }

        let tx_value = to_value(&output.amount);
        let amount_builder = output_builder.next()?;
        let built_output: csl::TransactionOutput = if tx_value.multiasset().is_some() {
            if tx_value.coin().is_zero() {
                amount_builder
                    .with_asset_and_min_required_coin_by_utxo_cost(
                        &tx_value.multiasset().unwrap(),
                        &csl::DataCost::new_coins_per_byte(&to_bignum(4310)),
                    )?
                    .build()?
            } else {
                amount_builder
                    .with_coin_and_asset(&tx_value.coin(), &tx_value.multiasset().unwrap())
                    .build()?
            }
        } else {
            amount_builder.with_coin(&tx_value.coin()).build()?
        };
        self.tx_builder.add_output(&built_output)?;
        Ok(())
    }

    fn add_collateral(
        &mut self,
        collateral_builder: &mut csl::TxInputsBuilder,
        collateral: PubKeyTxIn,
    ) -> Result<(), JsError> {
        collateral_builder.add_regular_input(
            &csl::Address::from_bech32(&collateral.tx_in.address.unwrap())?,
            &csl::TransactionInput::new(
                &csl::TransactionHash::from_hex(&collateral.tx_in.tx_hash)?,
                collateral.tx_in.tx_index,
            ),
            &to_value(&collateral.tx_in.amount.unwrap()),
        )?;
        Ok(())
    }

    fn add_reference_input(&mut self, ref_input: RefTxIn) -> Result<(), JsError> {
        let csl_ref_input = csl::TransactionInput::new(
            &csl::TransactionHash::from_hex(&ref_input.tx_hash)?,
            ref_input.tx_index,
        );
        self.tx_builder.add_reference_input(&csl_ref_input);
        Ok(())
    }

    fn add_pub_key_withdrawal(&mut self, withdrawal: PubKeyWithdrawal) -> Result<(), JsError> {
        self.tx_withdrawals_builder.add(
            &csl::RewardAddress::from_address(&csl::Address::from_bech32(&withdrawal.address)?)
                .unwrap(),
            &csl::BigNum::from_str(&withdrawal.coin.to_string())?,
        )?;
        Ok(())
    }

    fn add_plutus_withdrawal(&mut self, withdrawal: PlutusScriptWithdrawal) -> Result<(), JsError> {
        let script_source = withdrawal.script_source.unwrap();
        let redeemer = withdrawal.redeemer.unwrap();

        let csl_script: csl::PlutusScriptSource = match script_source {
            ScriptSource::ProvidedScriptSource(script) => {
                let language_version: csl::Language = match script.language_version {
                    LanguageVersion::V1 => csl::Language::new_plutus_v1(),
                    LanguageVersion::V2 => csl::Language::new_plutus_v2(),
                    LanguageVersion::V3 => csl::Language::new_plutus_v3(),
                };
                csl::PlutusScriptSource::new(&csl::PlutusScript::from_hex_with_version(
                    &script.script_cbor,
                    &language_version,
                )?)
            }
            ScriptSource::InlineScriptSource(script) => {
                let language_version: csl::Language = match script.language_version {
                    LanguageVersion::V1 => csl::Language::new_plutus_v1(),
                    LanguageVersion::V2 => csl::Language::new_plutus_v2(),
                    LanguageVersion::V3 => csl::Language::new_plutus_v3(),
                };
                csl::PlutusScriptSource::new_ref_input(
                    &csl::ScriptHash::from_hex(&script.spending_script_hash)?,
                    &csl::TransactionInput::new(
                        &csl::TransactionHash::from_hex(&script.tx_hash)?,
                        script.tx_index,
                    ),
                    &language_version,
                    script.script_size,
                )
            }
        };

        let csl_redeemer: csl::Redeemer = csl::Redeemer::new(
            &csl::RedeemerTag::new_spend(),
            &to_bignum(0),
            &csl::PlutusData::from_hex(&redeemer.data)?,
            &csl::ExUnits::new(
                &to_bignum(redeemer.ex_units.mem),
                &to_bignum(redeemer.ex_units.steps),
            ),
        );

        self.tx_withdrawals_builder.add_with_plutus_witness(
            &csl::RewardAddress::from_address(&csl::Address::from_bech32(&withdrawal.address)?)
                .unwrap(),
            &csl::BigNum::from_str(&withdrawal.coin.to_string())?,
            &csl::PlutusWitness::new_with_ref_without_datum(&csl_script, &csl_redeemer),
        )?;
        Ok(())
    }

    fn add_plutus_mint(
        &mut self,
        mint_builder: &mut csl::MintBuilder,
        mint: MintItem,
        index: u64,
    ) -> Result<(), JsError> {
        println!("6-2-1");
        let redeemer_info = mint.redeemer.unwrap();
        println!("6-2-2");
        let mint_redeemer = csl::Redeemer::new(
            &csl::RedeemerTag::new_mint(),
            &to_bignum(index),
            &csl::PlutusData::from_hex(&redeemer_info.data)?,
            &csl::ExUnits::new(
                &to_bignum(redeemer_info.ex_units.mem),
                &to_bignum(redeemer_info.ex_units.steps),
            ),
        );
        println!("6-2-3");
        let script_source_info = mint.script_source.unwrap();
        println!("6-2-4");
        let mint_script = match script_source_info {
            ScriptSource::InlineScriptSource(script) => {
                println!("6-2-5");
                let language_version: csl::Language = match script.language_version {
                    LanguageVersion::V1 => csl::Language::new_plutus_v1(),
                    LanguageVersion::V2 => csl::Language::new_plutus_v2(),
                    LanguageVersion::V3 => csl::Language::new_plutus_v3(),
                };
                println!("6-2-6");
                csl::PlutusScriptSource::new_ref_input(
                    &csl::ScriptHash::from_hex(mint.policy_id.as_str())?,
                    &csl::TransactionInput::new(
                        &csl::TransactionHash::from_hex(&script.tx_hash)?,
                        script.tx_index,
                    ),
                    &language_version,
                    script.script_size,
                )
            }
            ScriptSource::ProvidedScriptSource(script) => {
                println!("6-2-7");
                let language_version: csl::Language = match script.language_version {
                    LanguageVersion::V1 => csl::Language::new_plutus_v1(),
                    LanguageVersion::V2 => csl::Language::new_plutus_v2(),
                    LanguageVersion::V3 => csl::Language::new_plutus_v3(),
                };
                println!("6-2-8");
                csl::PlutusScriptSource::new(&csl::PlutusScript::from_hex_with_version(
                    script.script_cbor.as_str(),
                    &language_version,
                )?)
            }
        };

        println!("6-2-9");
        mint_builder.add_asset(
            &csl::MintWitness::new_plutus_script(&mint_script, &mint_redeemer),
            &csl::AssetName::new(hex::decode(mint.asset_name).unwrap())?,
            &csl::Int::new_i32(mint.amount.try_into().unwrap()),
        )?;
        println!("6-2-10");
        Ok(())
    }

    fn add_native_mint(
        &mut self,
        mint_builder: &mut csl::MintBuilder,
        mint: MintItem,
    ) -> Result<(), JsError> {
        let script_info = mint.script_source.unwrap();
        match script_info {
            ScriptSource::ProvidedScriptSource(script) => mint_builder.add_asset(
                &csl::MintWitness::new_native_script(&csl::NativeScriptSource::new(
                    &csl::NativeScript::from_hex(&script.script_cbor)?,
                )),
                &csl::AssetName::new(hex::decode(mint.asset_name).unwrap())?,
                &csl::Int::new_i32(mint.amount.try_into().unwrap()),
            )?,
            ScriptSource::InlineScriptSource(_) => {} // Err(csl::JsError::from_str(
                                                      //     "Native scripts cannot be referenced",
                                                      // )),
        };
        Ok(())
    }

    fn add_register_pool_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        register_pool: RegisterPool,
    ) -> Result<(), JsError> {
        let mut relays = csl::Relays::new();
        for relay in register_pool.pool_params.relays {
            match relay {
                Relay::SingleHostAddr(single_host_address_relay) => {
                    let ipv4_bytes: Option<csl::Ipv4> =
                        single_host_address_relay.ipv4.map(|ipv4_str| {
                            let addr: Ipv4Addr =
                                ipv4_str.parse().expect("ipv4 address parse failed");
                            csl::Ipv4::new(addr.octets().to_vec()).unwrap()
                        });

                    let ipv6_bytes: Option<csl::Ipv6> =
                        single_host_address_relay.ipv6.map(|ipv6_str| {
                            let addr: Ipv6Addr =
                                ipv6_str.parse().expect("ipv6 address parse failed");
                            csl::Ipv6::new(addr.octets().to_vec()).unwrap()
                        });
                    relays.add(&csl::Relay::new_single_host_addr(
                        &csl::SingleHostAddr::new(
                            single_host_address_relay.port,
                            ipv4_bytes,
                            ipv6_bytes,
                        ),
                    ));
                }
                Relay::SingleHostName(single_host_name_relay) => relays.add(
                    &csl::Relay::new_single_host_name(&csl::SingleHostName::new(
                        single_host_name_relay.port,
                        &csl::DNSRecordAorAAAA::new(single_host_name_relay.domain_name)?,
                    )),
                ),
                Relay::MultiHostName(multi_host_name_relay) => {
                    relays.add(&csl::Relay::new_multi_host_name(&csl::MultiHostName::new(
                        &csl::DNSRecordSRV::new(multi_host_name_relay.domain_name)?,
                    )))
                }
            }
        }

        let mut pool_owners = csl::Ed25519KeyHashes::new();
        for owner in register_pool.pool_params.owners {
            pool_owners.add(&csl::Ed25519KeyHash::from_hex(&owner)?);
        }

        certificate_builder.add(&csl::Certificate::new_pool_registration(
            &csl::PoolRegistration::new(&csl::PoolParams::new(
                &csl::Ed25519KeyHash::from_hex(&register_pool.pool_params.operator)?,
                &csl::VRFKeyHash::from_hex(&register_pool.pool_params.vrf_key_hash)?,
                &csl::BigNum::from_str(&register_pool.pool_params.pledge)?,
                &csl::BigNum::from_str(&register_pool.pool_params.cost)?,
                &csl::UnitInterval::new(
                    &csl::BigNum::from_str(&register_pool.pool_params.margin.0.to_string())?,
                    &csl::BigNum::from_str(&register_pool.pool_params.margin.1.to_string())?,
                ),
                &csl::RewardAddress::from_address(&csl::Address::from_bech32(
                    &register_pool.pool_params.reward_address,
                )?)
                .unwrap(),
                &pool_owners,
                &relays,
                register_pool.pool_params.metadata.map(|data| {
                    csl::PoolMetadata::new(
                        &csl::URL::new(data.url).unwrap(),
                        &csl::PoolMetadataHash::from_hex(&data.hash).unwrap(),
                    )
                }),
            )),
        ))?;
        Ok(())
    }

    fn add_register_stake_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        register_stake: RegisterStake,
    ) -> Result<(), JsError> {
        certificates_builder.add(&csl::Certificate::new_stake_registration(
            &csl::StakeRegistration::new(&csl::Credential::from_keyhash(
                &csl::Ed25519KeyHash::from_hex(&register_stake.stake_key_hash)?,
            )),
        ))?;
        Ok(())
    }

    fn add_delegate_stake_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        delegate_stake: DelegateStake,
    ) -> Result<(), JsError> {
        certificates_builder.add(&csl::Certificate::new_stake_delegation(
            &csl::StakeDelegation::new(
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &delegate_stake.stake_key_hash,
                )?),
                &csl::Ed25519KeyHash::from_hex(&delegate_stake.pool_id)?,
            ),
        ))?;
        Ok(())
    }

    fn add_deregister_stake_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        deregister_stake: DeregisterStake,
    ) -> Result<(), JsError> {
        certificates_builder.add(&csl::Certificate::new_stake_deregistration(
            &csl::StakeDeregistration::new(&csl::Credential::from_keyhash(
                &csl::Ed25519KeyHash::from_hex(&deregister_stake.stake_key_hash)?,
            )),
        ))?;
        Ok(())
    }

    fn add_retire_pool_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        retire_pool: RetirePool,
    ) -> Result<(), JsError> {
        certificates_builder.add(&csl::Certificate::new_pool_retirement(
            &csl::PoolRetirement::new(
                &csl::Ed25519KeyHash::from_hex(&retire_pool.pool_id)?,
                retire_pool.epoch,
            ),
        ))?;
        Ok(())
    }

    fn add_vote_delegation_cert(
        &mut self,
        certificates_builder: &mut csl::CertificatesBuilder,
        vote_delegation: VoteDelegation,
    ) -> Result<(), JsError> {
        certificates_builder.add(&csl::Certificate::new_vote_delegation(
            &csl::VoteDelegation::new(
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &vote_delegation.stake_key_hash,
                )?),
                &to_csl_drep(&vote_delegation.drep)?,
            ),
        ))?;
        Ok(())
    }

    fn add_stake_and_vote_delegation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        stake_and_vote_delegation: StakeAndVoteDelegation,
    ) -> Result<(), JsError> {
        certificate_builder.add(&csl::Certificate::new_stake_and_vote_delegation(
            &csl::StakeAndVoteDelegation::new(
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &stake_and_vote_delegation.stake_key_hash,
                )?),
                &csl::Ed25519KeyHash::from_hex(&stake_and_vote_delegation.pool_key_hash)?,
                &to_csl_drep(&stake_and_vote_delegation.drep)?,
            ),
        ))?;
        Ok(())
    }

    fn add_stake_registration_and_delegation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        stake_registration_and_delegation: StakeRegistrationAndDelegation,
    ) -> Result<(), JsError> {
        certificate_builder.add(&csl::Certificate::new_stake_registration_and_delegation(
            &csl::StakeRegistrationAndDelegation::new(
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &stake_registration_and_delegation.stake_key_hash,
                )?),
                &csl::Ed25519KeyHash::from_hex(&stake_registration_and_delegation.pool_key_hash)?,
                &to_bignum(stake_registration_and_delegation.coin),
            ),
        ))?;
        Ok(())
    }

    fn add_vote_registration_and_delgation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        vote_registration_and_delgation: VoteRegistrationAndDelegation,
    ) -> Result<(), JsError> {
        certificate_builder.add(&csl::Certificate::new_vote_registration_and_delegation(
            &csl::VoteRegistrationAndDelegation::new(
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &vote_registration_and_delgation.stake_key_hash,
                )?),
                &to_csl_drep(&vote_registration_and_delgation.drep)?,
                &to_bignum(vote_registration_and_delgation.coin),
            ),
        ))?;
        Ok(())
    }

    fn add_stake_vote_registration_and_delegation_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        stake_vote_registration_and_delegation: StakeVoteRegistrationAndDelegation,
    ) -> Result<(), JsError> {
        certificate_builder.add(
            &csl::Certificate::new_stake_vote_registration_and_delegation(
                &csl::StakeVoteRegistrationAndDelegation::new(
                    &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                        &stake_vote_registration_and_delegation.stake_key_hash,
                    )?),
                    &csl::Ed25519KeyHash::from_hex(
                        &stake_vote_registration_and_delegation.pool_key_hash,
                    )?,
                    &to_csl_drep(&stake_vote_registration_and_delegation.drep)?,
                    &to_bignum(stake_vote_registration_and_delegation.coin),
                ),
            ),
        )
    }

    fn add_committee_hot_auth_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        committee_hot_auth: CommitteeHotAuth,
    ) -> Result<(), JsError> {
        certificate_builder.add(&csl::Certificate::new_committee_hot_auth(
            &csl::CommitteeHotAuth::new(
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &committee_hot_auth.committee_cold_key_hash,
                )?),
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &committee_hot_auth.committee_hot_key_hash,
                )?),
            ),
        ))?;
        Ok(())
    }

    fn add_commitee_cold_resign_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        committee_cold_resign: CommitteeColdResign,
    ) -> Result<(), JsError> {
        let committee_cold_key = &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
            &committee_cold_resign.committee_cold_key_hash,
        )?);
        match committee_cold_resign.anchor {
            Some(anchor) => {
                certificate_builder.add(&csl::Certificate::new_committee_cold_resign(
                    &csl::CommitteeColdResign::new_with_anchor(
                        committee_cold_key,
                        &to_csl_anchor(&anchor)?,
                    ),
                ))?;
            }
            None => {
                certificate_builder.add(&csl::Certificate::new_committee_cold_resign(
                    &csl::CommitteeColdResign::new(committee_cold_key),
                ))?;
            }
        }
        Ok(())
    }

    fn add_drep_registration_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        drep_registration: DRepRegistration,
    ) -> Result<(), JsError> {
        certificate_builder.add(&csl::Certificate::new_drep_registration(
            &csl::DrepRegistration::new(
                &Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &drep_registration.voting_key_hash,
                )?),
                &to_bignum(drep_registration.coin),
            ),
        ))?;
        Ok(())
    }

    fn add_drep_deregistration_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        drep_deregistration: DRepDeregistration,
    ) -> Result<(), JsError> {
        certificate_builder.add(&csl::Certificate::new_drep_deregistration(
            &csl::DrepDeregistration::new(
                &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                    &drep_deregistration.voting_key_hash,
                )?),
                &to_bignum(drep_deregistration.coin),
            ),
        ))?;
        Ok(())
    }

    fn add_drep_update_cert(
        &mut self,
        certificate_builder: &mut csl::CertificatesBuilder,
        drep_update: DRepUpdate,
    ) -> Result<(), JsError> {
        match drep_update.anchor {
            Some(anchor) => certificate_builder.add(&csl::Certificate::new_drep_update(
                &csl::DrepUpdate::new_with_anchor(
                    &csl::Credential::from_keyhash(&csl::Ed25519KeyHash::from_hex(
                        &drep_update.voting_key_hash,
                    )?),
                    &to_csl_anchor(&anchor)?,
                ),
            )),
            None => certificate_builder.add(&csl::Certificate::new_drep_update(
                &csl::DrepUpdate::new(&csl::Credential::from_keyhash(
                    &csl::Ed25519KeyHash::from_hex(&drep_update.voting_key_hash)?,
                )),
            )),
        }?;
        Ok(())
    }

    fn add_invalid_before(&mut self, invalid_before: u64) {
        self.tx_builder
            .set_validity_start_interval_bignum(to_bignum(invalid_before));
    }

    fn add_invalid_hereafter(&mut self, invalid_hereafter: u64) {
        self.tx_builder
            .set_ttl_bignum(&to_bignum(invalid_hereafter));
    }

    fn add_change(
        &mut self,
        change_address: String,
        change_datum: Option<Datum>,
    ) -> Result<(), JsError> {
        if let Some(change_datum) = change_datum {
            self.tx_builder.add_change_if_needed_with_datum(
                &csl::Address::from_bech32(&change_address)?,
                &csl::OutputDatum::new_data(&csl::PlutusData::from_hex(change_datum.get_inner())?),
            )?;
        } else {
            self.tx_builder
                .add_change_if_needed(&csl::Address::from_bech32(&change_address)?)?;
        }
        Ok(())
    }

    fn add_signing_keys(&mut self, signing_keys: JsVecString) {
        self.tx_hex = sign_transaction(self.tx_hex.to_string(), signing_keys);
    }

    fn add_required_signature(&mut self, pub_key_hash: String) -> Result<(), JsError> {
        self.tx_builder
            .add_required_signer(&csl::Ed25519KeyHash::from_hex(&pub_key_hash)?);
        Ok(())
    }

    fn add_metadata(&mut self, metadata: Metadata) -> Result<(), JsError> {
        self.tx_builder
            .add_json_metadatum(&csl::BigNum::from_str(&metadata.tag)?, metadata.metadata)?;
        Ok(())
    }

    fn add_script_hash(&mut self) -> Result<(), JsError> {
        self.tx_builder
            .calc_script_data_hash(&csl::TxBuilderConstants::plutus_vasil_cost_models())?;
        Ok(())
    }

    fn build_tx(&mut self) -> Result<String, JsError> {
        let tx = self.tx_builder.build_tx().unwrap();
        self.tx_hex = tx.to_hex();
        Ok(self.tx_hex.to_string())
    }
}

impl Default for MeshCSL {
    fn default() -> Self {
        Self::new(None)
    }
}

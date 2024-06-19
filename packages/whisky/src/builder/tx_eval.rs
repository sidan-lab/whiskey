use async_trait::async_trait;
use cardano_serialization_lib::{self as csl};
use csl::JsError;
use pallas_primitives::alonzo::{Redeemer, RedeemerTag as PRedeemerTag};
use pallas_primitives::conway::PlutusV2Script;
use std::collections::HashMap;
use uplc::Fragment;

use csl::Address;
use pallas_codec::minicbor::Decoder;
use pallas_codec::utils::{Bytes, CborWrap, KeyValuePairs};
use pallas_primitives::babbage::{
    AssetName, Coin, CostMdls, DatumOption, PlutusData, PolicyId, PostAlonzoTransactionOutput,
    PseudoScript, ScriptRef, Value,
};
use sidan_csl_rs::core::constants::{get_v1_cost_models, get_v2_cost_models};
use sidan_csl_rs::core::tx_parser::{IMeshTxParser, MeshTxParser};
use sidan_csl_rs::model::{Action, Asset, Budget, RedeemerTag, UTxO, UtxoOutput};
use uplc::{
    tx::{eval_phase_two_raw, ResolvedInput},
    Hash, TransactionInput, TransactionOutput as PallasTransactionOutput,
};

use crate::service::IEvaluator;

pub struct MeshTxEvaluator {}

impl MeshTxEvaluator {
    pub fn new() -> Self {
        MeshTxEvaluator {}
    }
}

impl Default for MeshTxEvaluator {
    fn default() -> Self {
        MeshTxEvaluator::new()
    }
}

#[async_trait]
impl IEvaluator for MeshTxEvaluator {
    fn evaluate_tx_sync(
        &self,
        tx_hex: &str,
        inputs: &[UTxO],
        additional_txs: &[String],
    ) -> Result<Vec<Action>, JsError> {
        let tx_bytes = hex::decode(tx_hex).expect("Invalid tx hex");

        let tx_outs: Vec<UTxO> = additional_txs
            .iter()
            .flat_map(|tx| {
                let parsed_tx = MeshTxParser::new(tx);
                parsed_tx.get_tx_outs_utxo()
            })
            .collect();

        // combine inputs and tx_outs
        let all_inputs: Vec<UTxO> = inputs.iter().chain(tx_outs.iter()).cloned().collect();

        let pallas_utxos = to_pallas_utxos(&all_inputs).unwrap();

        let mut utxos = HashMap::new();
        for utxo in pallas_utxos.iter() {
            utxos.insert(
                TransactionInput::encode_fragment(&utxo.input).unwrap(),
                PallasTransactionOutput::encode_fragment(&utxo.output).unwrap(),
            );
        };

        let utxos_bytes = utxos.into_iter().collect::<Vec<(Vec<u8>, Vec<u8>)>>();

        eval_phase_two_raw(
            &tx_bytes,
            &utxos_bytes,
            &CostMdls::encode_fragment(&get_cost_mdls()).unwrap(),
            (10000000000, 14000000),
            (1596059091000, 4492800, 1000),
            false,
            |_r| (),
        )
        .map_err(|err| JsError::from_str(&format!("Error occurred during evaluation: {}", err)))
        .map(|reds| {
            reds.into_iter()
                .map(|red| {
                    let redeemer = Redeemer::decode_fragment(&red).unwrap();
                    Action {
                        index: redeemer.index,
                        budget: Budget {
                            mem: redeemer.ex_units.mem.into(),
                            steps: redeemer.ex_units.steps,
                        },
                        tag: match redeemer.tag {
                            PRedeemerTag::Spend => RedeemerTag::Spend,
                            PRedeemerTag::Mint => RedeemerTag::Mint,
                            PRedeemerTag::Cert => RedeemerTag::Cert,
                            PRedeemerTag::Reward => RedeemerTag::Reward,
                        },
                    }
                })
                .collect()
        })
    }
    async fn evaluate_tx(
        &self,
        tx_hex: &str,
        inputs: &[UTxO],
        additional_txs: &[String],
    ) -> Result<Vec<Action>, JsError> {
        let res = self.evaluate_tx_sync(tx_hex, inputs, additional_txs);
        println!("res: {:?}", res);
        res
    }
}

fn get_cost_mdls() -> CostMdls {
    CostMdls {
        plutus_v1: Some(get_v1_cost_models()),
        plutus_v2: Some(get_v2_cost_models()),
    }
}

fn to_pallas_utxos(utxos: &Vec<UTxO>) -> Result<Vec<ResolvedInput>, JsError> {
    let mut resolved_inputs = Vec::new();
    for utxo in utxos {
        let tx_hash: [u8; 32] = hex::decode(&utxo.input.tx_hash)
            .map_err(|err| JsError::from_str(&format!("Invalid tx hash found: {}", err)))?
            .try_into()
            .map_err(|_e| JsError::from_str("Invalid tx hash length found"))?;

        let resolved_input = ResolvedInput {
            input: TransactionInput {
                transaction_id: Hash::from(tx_hash),
                index: utxo.input.output_index.into(),
            },
            output: PallasTransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
                address: Bytes::from(
                    Address::from_bech32(&utxo.output.address)
                        .unwrap()
                        .to_bytes(),
                ),
                value: to_pallas_value(&utxo.output.amount)?,
                datum_option: to_pallas_datum(&utxo.output)?,
                script_ref: to_pallas_script_ref(&utxo.output)?,
            }),
        };
        resolved_inputs.push(resolved_input);
    }
    Ok(resolved_inputs)
}

// TODO: handle native and plutusV1 scripts
fn to_pallas_script_ref(utxo_output: &UtxoOutput) -> Result<Option<CborWrap<ScriptRef>>, JsError> {
    if let Some(script) = &utxo_output.script_ref {
        let script_bytes = hex::decode(script)
            .map_err(|err| JsError::from_str(&format!("Invalid script hex found: {}", err)))?;

        let unwrapped_bytes = Decoder::new(&script_bytes)
            .bytes()
            .map_err(|err| JsError::from_str(&format!("Invalid script hex found: {}", err)))?;

        Ok(Some(CborWrap(PseudoScript::PlutusV2Script(
            PlutusV2Script(unwrapped_bytes.to_vec().into()),
        ))))
    } else {
        Ok(None)
    }
}

fn to_pallas_datum(utxo_output: &UtxoOutput) -> Result<Option<DatumOption>, JsError> {
    if let Some(inline_datum) = &utxo_output.plutus_data {
        let csl_plutus_data =
            csl::PlutusData::from_json(inline_datum, csl::PlutusDatumSchema::DetailedSchema)
                .map_err(|err| JsError::from_str(&format!("Invalid plutus data found: {}", err)))?;

        let plutus_data_bytes = csl_plutus_data.to_bytes();
        let datum = CborWrap(
            PlutusData::decode_fragment(&plutus_data_bytes)
                .map_err(|_e| JsError::from_str("Invalid plutus data found"))?,
        );
        Ok(Some(DatumOption::Data(datum)))
    } else if let Some(datum_hash) = &utxo_output.data_hash {
        let datum_hash_bytes: [u8; 32] = hex::decode(datum_hash)
            .map_err(|err| JsError::from_str(&format!("Invalid datum hash found: {}", err)))?
            .try_into()
            .map_err(|_e| JsError::from_str("Invalid byte length of datum hash found"))?;
        Ok(Some(DatumOption::Hash(Hash::from(datum_hash_bytes))))
    } else {
        Ok(None)
    }
}

fn to_pallas_value(assets: &Vec<Asset>) -> Result<Value, JsError> {
    if assets.len() == 1 {
        match assets[0].unit().as_str() {
            "lovelace" => Ok(Value::Coin(assets[0].quantity().parse::<u64>().unwrap())),
            _ => Err(JsError::from_str("Invalid value")),
        }
    } else {
        to_pallas_multi_asset_value(assets)
    }
}

fn to_pallas_multi_asset_value(assets: &Vec<Asset>) -> Result<Value, JsError> {
    let mut coins: Coin = 0;
    let mut asset_mapping: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for asset in assets {
        if asset.unit() == "lovelace" || asset.unit().is_empty() {
            coins = asset.quantity().parse::<u64>().unwrap();
        } else {
            let asset_unit = asset.unit();
            let (policy_id, asset_name) = asset_unit.split_at(56);
            asset_mapping
                .entry(policy_id.to_string())
                .or_default()
                .push((asset_name.to_string(), asset.quantity().clone()))
        }
    }

    let mut multi_asset = Vec::new();
    for (policy_id, asset_list) in &asset_mapping {
        let policy_id_bytes: [u8; 28] = hex::decode(policy_id)
            .map_err(|err| JsError::from_str(&format!("Invalid policy id found: {}", err)))?
            .try_into()
            .map_err(|_e| JsError::from_str("Invalid length policy id found"))?;

        let policy_id = PolicyId::from(policy_id_bytes);
        let mut mapped_assets = Vec::new();
        for asset in asset_list {
            let (asset_name, asset_quantity) = asset;
            let asset_name_bytes =
                AssetName::from(hex::decode(asset_name).map_err(|err| {
                    JsError::from_str(&format!("Invalid asset name found: {}", err))
                })?);
            mapped_assets.push((asset_name_bytes, asset_quantity.parse::<u64>().unwrap()));
        }
        multi_asset.push((policy_id, KeyValuePairs::Def(mapped_assets)));
    }
    let pallas_multi_asset = KeyValuePairs::Def(multi_asset);
    Ok(Value::Multiasset(coins, pallas_multi_asset))
}

#[cfg(test)]
mod test {
    use pallas_codec::minicbor::Decoder;

    use super::*;
    use sidan_csl_rs::model::{Asset, UTxO, UtxoInput, UtxoOutput};
    // #[test]
    // fn test_eval() {
    // let result = tx_eval("84a80083825820879f68fef00fa676abcfba0396916299eddbf29e1103442aee031b383ee0f3ad01825820f51f44f1f16ceca8a96903b8f494a2da857a244066fa30c67e641d0f729fbde80c825820f51f44f1f16ceca8a96903b8f494a2da857a244066fa30c67e641d0f729fbde80d0181a300583910634a34d9c1ec5dd0cae61e4c86a4e85214bafdc80c57214fc80745b55ca749261aa3b17aa2cd4b026bc6566c4b14421d6083edce64ffe5cb011a1e6233b3028201d81858b1d8799fd8799fd87a9f581c103207deb2d24502f8438b5fcc556291877d5d365dafea4fcbd6d1d2ffd8799fd8799fd8799f581c5ca749261aa3b17aa2cd4b026bc6566c4b14421d6083edce64ffe5cbffffffffd8799fd87a9f581cfc259a2fc1a9fa6a6a902675aeba5415d0c33fc72049f5dc80e2b76effd8799fd8799fd8799f581c5ca749261aa3b17aa2cd4b026bc6566c4b14421d6083edce64ffe5cbffffffffd879801a000874101a1dcd6500ff021a0003c7cd09a00b5820b3d4c887f173ac071aff2e5ef18943311e9cad5cb7c4b578bebc82e1ff7628a50d818258203fbdf2b0b4213855dd9b87f7c94a50cf352ba6edfdded85ecb22cf9ceb75f814070e82581c36314aebecfbc929ee447dcb50fd690604eceae9403a298d9b1f9a54581c5ca51b304b1f79d92eada8c58c513e969458dcd27ce4f5bc47823ffa1285825820f51f44f1f16ceca8a96903b8f494a2da857a244066fa30c67e641d0f729fbde80d825820efe6fbbdd6b993d96883b96c572bfcaa0a4a138c83bd948dec1751d1bfda09b300825820879f68fef00fa676abcfba0396916299eddbf29e1103442aee031b383ee0f3ad01825820c4678d163c493e363f9e7fd9a310855ca6b4bdf508c73a463e1532d168dfb9a800825820f51f44f1f16ceca8a96903b8f494a2da857a244066fa30c67e641d0f729fbde80ca3008282582062f8f9ce8a5ed02fd67d6f2885b927874241a94b41e08fa5a99fc1e3bbca6453584051c0fe402fa75da2959e85eec47c45cf26149d8607be131ebaee4e86c92fe9e6ce9fa581192d7d644a216f4fbfd9fe94d6a827aba2ed0d81db6ff30095e8eb0d8258207f4747ca0c20a1e5c28716c4a10fffbcbe8fe6253cb427ae2f0e24d231a980845840e50b051bdad6c6c04d72bd5ed42f5bd981fa60c6709bacd24b5757f60418dabaa5455674e5135ab04d569f4db519ae1905ebbd47dca08df810a15c063ce0550503800583840000d87980821a000315621a04398879840001d87980821a000315621a04398879840002d87980821a000315621a04398879f5f6",
    //     &vec![UTxO {
    //         input: UtxoInput {
    //             tx_hash: "879f68fef00fa676abcfba0396916299eddbf29e1103442aee031b383ee0f3ad".to_string(),
    //             output_index: 1
    //         },
    //         output: UtxoOutput {
    //             address: "addr_test1zptl0h0ceq3d4tgrlkqgyv2n5cwez0juj9rm63uw8nxhpv6u5ayjvx4rk9a29n2tqf4uv4nvfv2yy8tqs0kuue8luh9sczalnn".to_string(),
    //             amount: vec![Asset {
    //                 unit: "lovelace".to_string(),
    //                 quantity: "5000000".to_string(),
    //             }],
    //             data_hash: None,
    //             plutus_data: Some(json!({
    //                 "constructor": 0,
    //                 "fields": []
    //              }).to_string()),
    //             script_hash: None,
    //             script_ref: Some("590a8c010000333323232323232323232232232232222323232533300f323232323232323232323232323232323232323253330233370e90000008a99981199b8748008c0880084c94ccc090cdc3a4000604600226464646464646464a66605e60640042646644646600200200444a66606800229444c8c94ccc0cccc0880180084cc010010004528181c0011bae30360013758602a605601c660626ea409ccc0c4dd480225eb80c94ccc0b4cdc3a4000002264646464a666068606e004264649319299981999b87480000044c8c94ccc0e0c0ec0084c9263253330363370e900000089919299981d981f00109924c60520022c607800260680042a66606c66e1d20020011323232323232533303f3042002149858dd6982000098200011bad303e001303e002375a607800260680042c60680022c607200260620062a66606666e1d200200115333036303100314985858c0c4008c08800c58c0d4004c0d4008c0cc004c0ac01858c0ac01458dd7181800098180011bae302e001302e002302c001302c002375c605400260440022c6464a66604a66e1d20003024001132323253330283370e900218138008981698130008b180b981298081812800981580098118008b19809000919b873330103756601c6046601c60460020429110048008dd6180618108020b0a99981199b87480080044c8c94ccc094cdc3a40046048008264a66604c66e1d20003025001132323253330293370e90021814000899191919299981699b87480080044c8c8c8c8c8c8c8c94ccc0d54ccc0d54ccc0d4010400c52808010a50100114a066660326eacc060c0c80540b0c05cc078c0c8c074c0c80352001323232323232323232533303c3370e9001181d80109919299981f002899b88010001003375a60840026074004002264a66607866e1d2002303b00213232533303e0050031337120200026eb4c108004c0e8008004528181d80119b8748008c0e8dd5181d80099bb00033330380014c103d87a80004c0103d87980003370e9001181c1baa303c001303c002303a0013032301e3032001301930310143302137586034606002605866e3c0040acdd7181a800981a8011bad3033001302b00314a06056004603e002605e002604e0022c6030604c6022604c002605800260480022c660186eb0c03cc08c01800458c0a4004c0840644c8c94ccc094cdc3a400460480082646464a66605066e1d200030270011323232323253330303033002132323232325333032533303253330325333032005100414a020062940400852808008a50323232323232323232533303a3370e9001181c80109919299981e002899b88001018003375a60800026070004002264a66607466e1d2002303900213232533303c0050031337120020306eb4c100004c0e0008004528181c80119b8748008c0e0dd5181c80099bb00033330360014c103d87a80004c0103d87980003370e9001181b1baa303a001303a00230380013030301b30300013017302f012323253330323370e900200089919299981a19b8748008c0cc0044c8c8c8c94ccc0ecc0f800854ccc0e0cdc7800819099b8700301414a02c6eb8c0f0004c0f0008dd6981d00098190008b181c00098180010a5030300013020302e004323253330313370e900200089919299981999b8748010c0c80044c8c94ccc0e0c0ec0084cdc78008178b1bae30390013031001163037001302f00214a0605e002603e605a6030605a00c66660266eacc048c0b003c098c04401120023301c3758602a605601c04e2c60620026644646600200200644a666064002297ae013232533303153330313375e6036605e00400e266e1cccc070dd5980d1817801014802a400429404cc0d4008cc0100100044cc010010004c0d8008c0d0004dd6180d98148061807000980a181418099814000981700098130008b198071bac30113025008001302b001302300416375a605200260420326042030604c002604c00460480026038026464a66603e66e1d2002301e00113232323253330233370e90010008980780189919299981299b8748000c0900044c8c8c94ccc0a0cdc3a4000002260286602a0106eb8c0b4c0980084c050cc054020dd7181698130011813000981580098118008b1814800981080118108009805180f8021bae3025001301d001163008301c001230223023302330233023001222232533302300114a0264a666048002264646464a66604aa66604a66e3c0040244cdc78010040a5013370e00600e2940dd718148019bae30283029002375a604e605060500026eb0c09800852818130009919198008008011129998128008a5eb804c8ccc888c8cc00400400c894ccc0ac004400c4c8cc0b4dd3998169ba90063302d37526eb8c0a8004cc0b4dd41bad302b0014bd7019801801981780118168009bae30240013756604a002660060066052004604e002646600200200a44a666048002297adef6c6013232323253330253371e911000021003133029337606ea4008dd3000998030030019bab3026003375c60480046050004604c0024604060426042604260426042604260420024466012004466ebcc018c0680040088c078c07cc07cc07cc07cc07cc07cc07cc07c0048c074c0780048c070004888c8c8c94ccc06ccdc3a40040022900009bad30203019002301900132533301a3370e90010008a60103d87a8000132323300100100222533302000114c103d87a800013232323253330213371e014004266e95200033025375000297ae0133006006003375a60440066eb8c080008c090008c088004dd5980f980c001180c000991980080080211299980e8008a6103d87a8000132323232533301e3371e010004266e95200033022374c00297ae01330060060033756603e0066eb8c074008c084008c07c0048dca0009119b8a00200122323300100100322533301900114c0103d87a8000132325333018300500213374a90001980e00125eb804cc010010004c074008c06c00488c8cc00400400c894ccc06000452809919299980b99b8f00200514a226600800800260380046eb8c0680048c058c05cc05c0048c94ccc044cdc3a400000226464a66602c60320042930b1bae3017001300f002153330113370e900100089919299980b180c8010a4c2c6eb8c05c004c03c00858c03c0045261365632533300f3370e90000008a99980918068028a4c2c2a66601e66e1d20020011323253330143017002132498c94ccc048cdc3a4000002264646464a66603260380042649319299980b99b87480000044c8c94ccc070c07c00852616375c603a002602a0082c602a0062c6eb4c068004c068008c060004c04000858c04000458c054004c03401454ccc03ccdc3a400800226464a666028602e0042930b1bad3015001300d00516300d0043001004232533300e3370e90000008a99980898060010a4c2c2a66601c66e1d200200113232323253330153018002149858dd7180b000980b0011bad3014001300c0021533300e3370e9002000899192999809980b0010a4c2c6eb8c050004c03000858c030004dd70009bae001375c0024600a6ea80048c00cdd5000ab9a5573aaae7955cfaba05742ae8930011e581ce6e5285a878161c101a59b4e36f1f99e5e464d30f510be3ee34f907f004c011e581cd161d64eef0eeb59f9124f520f8c8f3b717ed04198d54c8b17e604ae004c011e581c2291f67ee643db1a830734bd54d39022c5d1f990682e689c95d8fed00001".to_string()),
    //         }
    //     },
    //     UTxO {
    //         input: UtxoInput {
    //             tx_hash: "f51f44f1f16ceca8a96903b8f494a2da857a244066fa30c67e641d0f729fbde8".to_string(),
    //             output_index: 12
    //         },
    //         output: UtxoOutput {
    //             address: "addr_test1zqgryp77ktfy2qhcgw94lnz4v2gcwl2axew6l6j0e0tdr5ju5ayjvx4rk9a29n2tqf4uv4nvfv2yy8tqs0kuue8luh9sl4yffp".to_string(),
    //             amount: vec![Asset {
    //                 unit: "lovelace".to_string(),
    //                 quantity: "5000000".to_string(),
    //             }],
    //             data_hash: None,
    //             plutus_data: Some(json!({
    //                 "constructor": 0,
    //                 "fields": []
    //              }).to_string()),
    //             script_hash: None,
    //             script_ref: None,
    //         }
    //     },
    //     UTxO {
    //         input: UtxoInput {
    //             tx_hash: "f51f44f1f16ceca8a96903b8f494a2da857a244066fa30c67e641d0f729fbde8".to_string(),
    //             output_index: 13
    //         },
    //         output: UtxoOutput {
    //             address: "addr_test1qqmrzjhtanauj20wg37uk58adyrqfm82a9qr52vdnv0e54r42v0mu8ngky0f5yxmh3wl3z0da2fryk59kavth0u8xhvsufgmc8".to_string(),
    //             amount: vec![Asset {
    //                 unit: "lovelace".to_string(),
    //                 quantity: "143618073".to_string(),
    //             }, Asset {
    //                 unit: "5066154a102ee037390c5236f78db23239b49c5748d3d349f3ccf04b55534458".to_string(),
    //                 quantity: "499000000".to_string(),
    //             }],
    //             data_hash: None,
    //             plutus_data: Some(json!({
    //                 "constructor": 0,
    //                 "fields": []
    //              }).to_string()),
    //             script_hash: None,
    //             script_ref: None,
    //         }
    //     },
    //     UTxO {
    //         input: UtxoInput {
    //             tx_hash: "efe6fbbdd6b993d96883b96c572bfcaa0a4a138c83bd948dec1751d1bfda09b3".to_string(),
    //             output_index: 0
    //         },
    //         output: UtxoOutput {
    //             address: "addr_test1zqjmsmh2sjjy508e3068pck6lgp23k2msypgc52cxcgzjlju5ayjvx4rk9a29n2tqf4uv4nvfv2yy8tqs0kuue8luh9s5cdt49".to_string(),
    //             amount: vec![Asset {
    //                 unit: "lovelace".to_string(),
    //                 quantity: "1909330".to_string(),
    //             }, Asset {
    //                 unit: "e6e5285a878161c101a59b4e36f1f99e5e464d30f510be3ee34f907f".to_string(),
    //                 quantity: "1".to_string(),
    //             }],
    //             data_hash: None,
    //             plutus_data: Some(json!({
    //                 "constructor": 0,
    //                 "fields": [
    //                    {
    //                       "bytes": "e6e5285a878161c101a59b4e36f1f99e5e464d30f510be3ee34f907f"
    //                    },
    //                    {
    //                       "constructor": 0,
    //                       "fields": [
    //                          {
    //                             "constructor": 1,
    //                             "fields": [
    //                                {
    //                                   "bytes": "25b86eea84a44a3cf98bf470e2dafa02a8d95b81028c51583610297e"
    //                                }
    //                             ]
    //                          },
    //                          {
    //                             "constructor": 0,
    //                             "fields": [
    //                                {
    //                                   "constructor": 0,
    //                                   "fields": [
    //                                      {
    //                                         "constructor": 0,
    //                                         "fields": [
    //                                            {
    //                                               "bytes": "5ca749261aa3b17aa2cd4b026bc6566c4b14421d6083edce64ffe5cb"
    //                                            }
    //                                         ]
    //                                      }
    //                                   ]
    //                                }
    //                             ]
    //                          }
    //                       ]
    //                    },
    //                    {
    //                       "bytes": "5ca51b304b1f79d92eada8c58c513e969458dcd27ce4f5bc47823ffa"
    //                    },
    //                    {
    //                       "bytes": "bbb1a36cc3e076d689176e77374ca26d4e09047c9d9dbd10ab0dcdae"
    //                    }
    //                 ]
    //              }).to_string()),
    //             script_hash: None,
    //             script_ref: None,
    //         }
    //     },
    //     UTxO {
    //         input: UtxoInput {
    //             tx_hash: "c4678d163c493e363f9e7fd9a310855ca6b4bdf508c73a463e1532d168dfb9a8".to_string(),
    //             output_index: 0
    //         },
    //         output: UtxoOutput {
    //             address: "addr_test1zqgryp77ktfy2qhcgw94lnz4v2gcwl2axew6l6j0e0tdr5ju5ayjvx4rk9a29n2tqf4uv4nvfv2yy8tqs0kuue8luh9sl4yffp".to_string(),
    //             amount: vec![Asset {
    //                 unit: "lovelace".to_string(),
    //                 quantity: "12684330".to_string(),
    //             }],
    //             data_hash: None,
    //             plutus_data: None,
    //             script_hash: None,
    //             script_ref: Some("590a8c010000333323232323232323232232232232222323232533300f323232323232323232323232323232323232323253330233370e90000008a99981199b8748008c0880084c94ccc090cdc3a4000604600226464646464646464a66605e60640042646644646600200200444a66606800229444c8c94ccc0cccc0880180084cc010010004528181c0011bae30360013758602a605601c660626ea409ccc0c4dd480225eb80c94ccc0b4cdc3a4000002264646464a666068606e004264649319299981999b87480000044c8c94ccc0e0c0ec0084c9263253330363370e900000089919299981d981f00109924c60520022c607800260680042a66606c66e1d20020011323232323232533303f3042002149858dd6982000098200011bad303e001303e002375a607800260680042c60680022c607200260620062a66606666e1d200200115333036303100314985858c0c4008c08800c58c0d4004c0d4008c0cc004c0ac01858c0ac01458dd7181800098180011bae302e001302e002302c001302c002375c605400260440022c6464a66604a66e1d20003024001132323253330283370e900218138008981698130008b180b981298081812800981580098118008b19809000919b873330103756601c6046601c60460020429110048008dd6180618108020b0a99981199b87480080044c8c94ccc094cdc3a40046048008264a66604c66e1d20003025001132323253330293370e90021814000899191919299981699b87480080044c8c8c8c8c8c8c8c94ccc0d54ccc0d54ccc0d4010400c52808010a50100114a066660326eacc060c0c80540b0c05cc078c0c8c074c0c80352001323232323232323232533303c3370e9001181d80109919299981f002899b88010001003375a60840026074004002264a66607866e1d2002303b00213232533303e0050031337120200026eb4c108004c0e8008004528181d80119b8748008c0e8dd5181d80099bb00033330380014c103d87a80004c0103d87980003370e9001181c1baa303c001303c002303a0013032301e3032001301930310143302137586034606002605866e3c0040acdd7181a800981a8011bad3033001302b00314a06056004603e002605e002604e0022c6030604c6022604c002605800260480022c660186eb0c03cc08c01800458c0a4004c0840644c8c94ccc094cdc3a400460480082646464a66605066e1d200030270011323232323253330303033002132323232325333032533303253330325333032005100414a020062940400852808008a50323232323232323232533303a3370e9001181c80109919299981e002899b88001018003375a60800026070004002264a66607466e1d2002303900213232533303c0050031337120020306eb4c100004c0e0008004528181c80119b8748008c0e0dd5181c80099bb00033330360014c103d87a80004c0103d87980003370e9001181b1baa303a001303a00230380013030301b30300013017302f012323253330323370e900200089919299981a19b8748008c0cc0044c8c8c8c94ccc0ecc0f800854ccc0e0cdc7800819099b8700301414a02c6eb8c0f0004c0f0008dd6981d00098190008b181c00098180010a5030300013020302e004323253330313370e900200089919299981999b8748010c0c80044c8c94ccc0e0c0ec0084cdc78008178b1bae30390013031001163037001302f00214a0605e002603e605a6030605a00c66660266eacc048c0b003c098c04401120023301c3758602a605601c04e2c60620026644646600200200644a666064002297ae013232533303153330313375e6036605e00400e266e1cccc070dd5980d1817801014802a400429404cc0d4008cc0100100044cc010010004c0d8008c0d0004dd6180d98148061807000980a181418099814000981700098130008b198071bac30113025008001302b001302300416375a605200260420326042030604c002604c00460480026038026464a66603e66e1d2002301e00113232323253330233370e90010008980780189919299981299b8748000c0900044c8c8c94ccc0a0cdc3a4000002260286602a0106eb8c0b4c0980084c050cc054020dd7181698130011813000981580098118008b1814800981080118108009805180f8021bae3025001301d001163008301c001230223023302330233023001222232533302300114a0264a666048002264646464a66604aa66604a66e3c0040244cdc78010040a5013370e00600e2940dd718148019bae30283029002375a604e605060500026eb0c09800852818130009919198008008011129998128008a5eb804c8ccc888c8cc00400400c894ccc0ac004400c4c8cc0b4dd3998169ba90063302d37526eb8c0a8004cc0b4dd41bad302b0014bd7019801801981780118168009bae30240013756604a002660060066052004604e002646600200200a44a666048002297adef6c6013232323253330253371e911000021003133029337606ea4008dd3000998030030019bab3026003375c60480046050004604c0024604060426042604260426042604260420024466012004466ebcc018c0680040088c078c07cc07cc07cc07cc07cc07cc07cc07c0048c074c0780048c070004888c8c8c94ccc06ccdc3a40040022900009bad30203019002301900132533301a3370e90010008a60103d87a8000132323300100100222533302000114c103d87a800013232323253330213371e014004266e95200033025375000297ae0133006006003375a60440066eb8c080008c090008c088004dd5980f980c001180c000991980080080211299980e8008a6103d87a8000132323232533301e3371e010004266e95200033022374c00297ae01330060060033756603e0066eb8c074008c084008c07c0048dca0009119b8a00200122323300100100322533301900114c0103d87a8000132325333018300500213374a90001980e00125eb804cc010010004c074008c06c00488c8cc00400400c894ccc06000452809919299980b99b8f00200514a226600800800260380046eb8c0680048c058c05cc05c0048c94ccc044cdc3a400000226464a66602c60320042930b1bae3017001300f002153330113370e900100089919299980b180c8010a4c2c6eb8c05c004c03c00858c03c0045261365632533300f3370e90000008a99980918068028a4c2c2a66601e66e1d20020011323253330143017002132498c94ccc048cdc3a4000002264646464a66603260380042649319299980b99b87480000044c8c94ccc070c07c00852616375c603a002602a0082c602a0062c6eb4c068004c068008c060004c04000858c04000458c054004c03401454ccc03ccdc3a400800226464a666028602e0042930b1bad3015001300d00516300d0043001004232533300e3370e90000008a99980898060010a4c2c2a66601c66e1d200200113232323253330153018002149858dd7180b000980b0011bad3014001300c0021533300e3370e9002000899192999809980b0010a4c2c6eb8c050004c03000858c030004dd70009bae001375c0024600a6ea80048c00cdd5000ab9a5573aaae7955cfaba05742ae8930011e581ce6e5285a878161c101a59b4e36f1f99e5e464d30f510be3ee34f907f004c011e581c36314aebecfbc929ee447dcb50fd690604eceae9403a298d9b1f9a54004c011e581c2291f67ee643db1a830734bd54d39022c5d1f990682e689c95d8fed00001".to_string()),
    //         }
    //     }]).unwrap();
    // }

    #[test]
    fn test_eval() {
        let evaluator = MeshTxEvaluator::new();
        let result = evaluator.evaluate_tx_sync(
            "84a80082825820604943e070ffbf81cc09bb2942029f5f5361108a3c0b96a7309e6aa70370ad9800825820604943e070ffbf81cc09bb2942029f5f5361108a3c0b96a7309e6aa70370ad98010d81825820604943e070ffbf81cc09bb2942029f5f5361108a3c0b96a7309e6aa70370ad9801128182582004b9070a30bd63abaaf59a3c48a1575c4127bb0edb00ecd5141fd18a85c721aa000181a200581d601fd5bab167338971d92b4d8f0bdf57d889903e6e934e7ea38c7dadf1011b00000002529898c810a200581d601fd5bab167338971d92b4d8f0bdf57d889903e6e934e7ea38c7dadf1011b0000000252882db4111a000412f1021a0002b74b0b5820775d0cf3c95993f6210e4410e92f72ebc3942ce9c1433694749aa239e5d13387a200818258201557f444f3ae6e61dfed593ae15ec8dbd57b8138972bf16fde5b4c559f41549b5840729f1f14ef05b7cf9b0d7583e6777674f80ae64a35bbd6820cc3c82ddf0412ca1d751b7d886eece3c6e219e1c5cc9ef3d387a8d2078f47125d54b474fbdfbd0105818400000182190b111a000b5e35f5f6",
          &vec![UTxO {
              input: UtxoInput {
                  tx_hash: "604943e070ffbf81cc09bb2942029f5f5361108a3c0b96a7309e6aa70370ad98".to_string(),
                  output_index: 0
              },
              output: UtxoOutput {
                  address: "addr_test1wzlwsgq97vchypqzk8u8lz30w932tvx7akcj7csm02scl7qlghd97".to_string(),
                  amount: vec![Asset::new_from_str("lovelace", "986990")],
                  data_hash: None,
                  plutus_data: Some(serde_json::json!({
                      "constructor": 0,
                      "fields": []
                  }).to_string()),
                  script_hash: None,
                  script_ref: None,
              }
          },
          UTxO {
              input: UtxoInput {
                  tx_hash: "604943e070ffbf81cc09bb2942029f5f5361108a3c0b96a7309e6aa70370ad98".to_string(),
                  output_index: 1
              },
              output: UtxoOutput {
                  address: "addr_test1vq0atw43vuecjuwe9dxc7z7l2lvgnyp7d6f5ul4r3376mug8v67h5".to_string(),
                  amount: vec![Asset::new_from_str("lovelace", "9974857893")],
                  data_hash: None,
                  plutus_data: None,
                  script_hash: None,
                  script_ref: None,
              }
          },
          UTxO {
              input: UtxoInput {
                  tx_hash: "04b9070a30bd63abaaf59a3c48a1575c4127bb0edb00ecd5141fd18a85c721aa".to_string(),
                  output_index: 0
              },
              output: UtxoOutput {
                  address: "addr_test1wzlwsgq97vchypqzk8u8lz30w932tvx7akcj7csm02scl7qlghd97".to_string(),
                  amount: vec![Asset::new_from_str("lovelace", "986990")],
                  data_hash: None,
                  plutus_data: None,
                  script_hash: None,
                  script_ref: Some("5655010000322223253330054a229309b2b1bad0025735".to_string())
              }
          }],
          &[]
      );

        let redeemers = result.unwrap();
        let mut redeemer_json = serde_json::Map::new();
        for redeemer in redeemers {
            redeemer_json.insert("index".to_string(), redeemer.index.to_string().into());
            let mut ex_unit_json = serde_json::Map::new();
            ex_unit_json.insert("mem".to_string(), redeemer.budget.mem.into());
            ex_unit_json.insert("steps".to_string(), redeemer.budget.steps.into());
            redeemer_json.insert(
                "ex_units".to_string(),
                serde_json::Value::Object(ex_unit_json),
            );
        }
        assert_eq!(
            serde_json::json!({"ex_units":{"mem":2833,"steps":745013},"index":"0"}).to_string(),
            serde_json::json!(redeemer_json).to_string()
        )
    }

    #[test]
    fn test_cbor() {
        let script_bytes = hex::decode("5655010000322223253330054a229309b2b1bad0025735").unwrap();
        let decoded_bytes = Decoder::new(&script_bytes).bytes().unwrap();
        assert_eq!(
            hex::decode("55010000322223253330054a229309b2b1bad0025735").unwrap(),
            decoded_bytes
        );
    }
}

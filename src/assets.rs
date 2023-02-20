use apollo_cw_asset::{Asset, AssetInfo, AssetList};
use cosmwasm_std::{
    attr, to_binary, Addr, Api, Coin, CosmosMsg, Env, Event, MessageInfo, Response, StdError,
    StdResult, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ExecuteMsg};

/// Create an AssetList from a `Vec<Coin>` and an optional `Vec<Cw20Coin>`.
/// Removes duplicates from each of the inputs.
pub fn to_asset_list(
    api: &dyn Api,
    coins: Option<&Vec<Coin>>,
    cw20s: Option<&Vec<Cw20Coin>>,
) -> StdResult<AssetList> {
    let mut assets = AssetList::new();

    if let Some(coins) = coins {
        for coin in coins {
            assets.add(&coin.into())?;
        }
    }

    if let Some(cw20s) = cw20s {
        for cw20 in cw20s {
            assets.add(&Asset::new(
                AssetInfo::Cw20(api.addr_validate(&cw20.address)?),
                cw20.amount,
            ))?;
        }
    }
    Ok(assets)
}

/// Converts an `AssetList` into a `Vec<Coin>` and a `Vec<Cw20Coin>`.
pub fn separate_natives_and_cw20s(assets: &AssetList) -> (Vec<Coin>, Vec<Cw20Coin>) {
    let mut coins = vec![];
    let mut cw20s = vec![];

    for asset in assets.into_iter() {
        match &asset.info {
            AssetInfo::Native(token) => {
                coins.push(Coin {
                    denom: token.to_string(),
                    amount: asset.amount,
                });
            }
            AssetInfo::Cw20(addr) => {
                cw20s.push(Cw20Coin {
                    address: addr.to_string(),
                    amount: asset.amount,
                });
            }
        }
    }

    // Cosmos SDK coins need to be sorted and currently wasmd does not sort
    // CosmWasm coins when converting into SDK coins.
    coins.sort_by(|a, b| a.denom.cmp(&b.denom));

    (coins, cw20s)
}

/// Assert that a specific native token in the form of an `Asset` was sent to
/// the contract.
pub fn assert_native_token_received(info: &MessageInfo, asset: &Asset) -> StdResult<()> {
    let coin: Coin = asset.try_into()?;

    if !info.funds.contains(&coin) {
        return Err(StdError::generic_err(format!(
            "Assert native token received failed for asset: {}",
            asset
        )));
    }
    Ok(())
}

/// Assert that all assets in the `AssetList` are native tokens, and that all of
/// them were also sent in the correct amount in `info.funds`.
/// Does not error if there are additional native tokens in `info.funds` that
/// are not in the `AssetList`.
///
/// ### Returns
/// Returns a `Vec<Coin>` with all the native tokens in `info.funds`.
///
/// ### Errors
/// Returns an error if any of the assets in the `AssetList` are not native
/// tokens.
/// Returns an error if any of the native tokens in the `AssetList` were not
/// sent in `info.funds`.
pub fn assert_native_tokens_received(
    info: &MessageInfo,
    assets: &AssetList,
) -> StdResult<Vec<Coin>> {
    let coins = assert_only_native_coins(assets)?;
    for coin in &coins {
        if !info.funds.contains(&coin) {
            return Err(StdError::generic_err(format!(
                "Assert native token received failed for asset: {}",
                coin
            )));
        }
    }
    Ok(info.funds.clone())
}

/// Calls TransferFrom on an Asset if it is a Cw20. If it is a native we just
/// assert that the native token was already sent to the contract.
///
/// ### Returns
/// Returns a response with the transfer_from message if the asset is a Cw20.
/// Returns an empty response if the asset is a native token.
pub fn receive_asset(info: &MessageInfo, env: &Env, asset: &Asset) -> StdResult<Response> {
    let event = Event::new("apollo/utils/assets").add_attributes(vec![
        attr("action", "receive_asset"),
        attr("asset", asset.to_string()),
    ]);
    match &asset.info {
        AssetInfo::Cw20(_coin) => {
            let msg =
                asset.transfer_from_msg(info.sender.clone(), env.contract.address.to_string())?;
            Ok(Response::new().add_message(msg).add_event(event))
        }
        AssetInfo::Native(_token) => {
            //Here we just assert that the native token was sent with the contract call
            assert_native_token_received(info, asset)?;
            Ok(Response::new().add_event(event))
        }
    }
}

/// Returns an `Option` with a [`CosmosMsg`] that transfers the asset
/// to `env.contract.address`. If the asset is a native token, it checks
/// the that the funds were recieved in `info.funds` and returns `None`.
fn receive_asset_msg(info: &MessageInfo, env: &Env, asset: &Asset) -> StdResult<Option<CosmosMsg>> {
    match &asset.info {
        AssetInfo::Cw20(_coin) => {
            Some(asset.transfer_from_msg(info.sender.clone(), env.contract.address.to_string()))
                .transpose()
        }
        AssetInfo::Native(_token) => {
            //Here we just assert that the native token was sent with the contract call
            assert_native_token_received(info, asset)?;
            Ok(None)
        }
    }
}

/// Verifies that all native tokens were a sent in `info.funds` and returns
/// a `Response` with a messages that transfers all Cw20 tokens to
/// `env.contract.address`.
pub fn receive_assets(info: &MessageInfo, env: &Env, assets: &AssetList) -> StdResult<Response> {
    let event = Event::new("apollo/utils/assets").add_attributes(vec![
        attr("action", "receive_assets"),
        attr("assets", assets.to_string()),
    ]);
    let msgs = assets
        .into_iter()
        .map(|asset| receive_asset_msg(info, env, asset))
        .collect::<StdResult<Vec<Option<_>>>>()?
        .into_iter()
        .filter_map(|msg| msg)
        .collect::<Vec<_>>();

    Ok(Response::new().add_messages(msgs).add_event(event))
}

/// Assert that all assets in the `AssetList` are native tokens.
///
/// ### Returns
/// Returns an error if any of the assets are not native tokens.
/// Returns a `StdResult<Vec<Coin>>` containing the assets as coins if they are
/// all native tokens.
pub fn assert_only_native_coins(assets: &AssetList) -> StdResult<Vec<Coin>> {
    assets
        .into_iter()
        .map(assert_native_coin)
        .collect::<StdResult<Vec<Coin>>>()
}

/// Assert that an asset is a native token.
///
/// ### Returns
/// Returns an error if the asset is not a native token.
/// Returns a `StdResult<Coin>` containing the asset as a coin if it is a native
/// token.
pub fn assert_native_coin(asset: &Asset) -> StdResult<Coin> {
    match asset.info {
        AssetInfo::Native(_) => asset.try_into(),
        _ => Err(StdError::generic_err("Asset is not a native token")),
    }
}

/// Assert that an AssetInfo is a native token.
///
/// ### Returns
/// Returns an error if the AssetInfo is not a native token.
/// Returns a `StdResult<String>` containing the denom if it is a native token.
pub fn assert_native_asset_info(asset_info: &AssetInfo) -> StdResult<String> {
    match asset_info {
        AssetInfo::Native(denom) => Ok(denom.clone()),
        _ => Err(StdError::generic_err("AssetInfo is not a native token")),
    }
}

/// Merge duplicates of assets in an `AssetList`.
///
/// ### Returns
/// Returns the asset list with all duplicates merged.
pub fn merge_assets<'a, A: Into<&'a AssetList>>(assets: A) -> StdResult<AssetList> {
    let asset_list = assets.into();
    let mut merged = AssetList::new();
    for asset in asset_list {
        merged.add(asset)?;
    }
    Ok(merged)
}

/// Separate native tokens and Cw20's in an `AssetList` and return messages
/// for increasing allowance for the Cw20's.
///
/// ### Returns
/// Returns a `StdResult<(Vec<CosmosMsg>, Vec<Coin>)>` containing the messages
/// for increasing allowance and the native tokens.
pub fn increase_allowance_msgs(
    env: &Env,
    assets: &AssetList,
    recipient: Addr,
) -> StdResult<(Vec<CosmosMsg>, Vec<Coin>)> {
    let (funds, cw20s) = separate_natives_and_cw20s(assets);
    let msgs: Vec<CosmosMsg> = cw20s
        .into_iter()
        .map(|x| {
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: x.address,
                msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                    spender: recipient.to_string(),
                    amount: x.amount,
                    expires: Some(cw20::Expiration::AtHeight(env.block.height + 1)),
                })?,
                funds: vec![],
            }))
        })
        .collect::<StdResult<Vec<_>>>()?;
    Ok((msgs, funds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use apollo_cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetList};
    use cosmwasm_std::testing::{mock_env, mock_info, MockApi};
    use cosmwasm_std::CosmosMsg::Wasm;
    use cosmwasm_std::ReplyOn::Never;
    use cosmwasm_std::StdError::GenericErr;
    use cosmwasm_std::WasmMsg::Execute;
    use cosmwasm_std::{to_binary, Addr, Coin, SubMsg, Uint128};
    use cw20::{Cw20ExecuteMsg, Expiration};
    use test_case::test_case;

    #[test_case(
        vec![Coin::new(1000, "uosmo"), Coin::new(1000, "uatom")].into(),
        vec![Coin::new(1000, "uosmo"), Coin::new(1000, "uatom")]
        => Ok(());
        "Only native tokens, all sent")]
    #[test_case(
        vec![Coin::new(1000, "uosmo"), Coin::new(1000, "uatom")].into(),
        vec![Coin::new(1000, "uosmo"), Coin::new(10, "uatom")]
        => Err(StdError::generic_err("Assert native token received failed for asset: 1000uatom"));
        "Only native tokens, some not sent")]
    #[test_case(
        vec![Coin::new(1000, "uosmo"), Coin::new(1000, "uatom")].into(),
        vec![Coin::new(1000, "uosmo")]
        => Err(StdError::generic_err("Assert native token received failed for asset: 1000uatom"));
        "Only native tokens, one missing coin")]
    #[test_case(
        vec![Asset::new(AssetInfo::Native("uosmo".into()), 1000u128), Asset::new(AssetInfo::cw20(Addr::unchecked("apollo")), 1000u128)].into(),
        vec![Coin::new(1000, "uosmo")]
        => Err(StdError::generic_err("Asset is not a native token"));
        "Mixed native and cw20 tokens")]
    #[test_case(
        AssetList::new(),
        vec![]
        => Ok(());
        "Empty asset list, empty funds")]
    #[test_case(
        vec![Coin::new(1000, "uosmo")].into(),
        vec![]
        => Err(StdError::generic_err("Assert native token received failed for asset: 1000uosmo"));
        "1 native token in asset list, empty funds")]
    #[test_case(
        AssetList::new(),
        vec![Coin::new(1000, "uosmo")]
        => Ok(());
        "Empty asset list, 1 native token in funds")]
    fn test_assert_native_tokens_received(assets: AssetList, funds: Vec<Coin>) -> StdResult<()> {
        let info = mock_info("addr", &funds);
        assert_native_tokens_received(&info, &assets)?;
        Ok(())
    }

    #[test]
    fn test_receive_asset_cw20() {
        let funds = vec![Coin::new(1000, "uosmo")];
        let info = mock_info("addr", &funds);
        let env = mock_env();
        let asset = Asset::new(AssetInfo::cw20(Addr::unchecked("apollo")), 1000u128);
        let result = receive_asset(&info, &env, &asset);
        assert!(result.is_ok());
        let response = result.unwrap();

        let expected_events = vec![Event::new("apollo/utils/assets").add_attributes(vec![
            attr("action", "receive_asset"),
            attr("asset", "apollo:1000"),
        ])];

        let expected_messages = vec![SubMsg {
            id: 0,
            msg: Wasm(Execute {
                contract_addr: String::from("apollo"),
                msg: to_binary(
                    &(Cw20ExecuteMsg::TransferFrom {
                        owner: String::from("addr"),
                        recipient: String::from("cosmos2contract"),
                        amount: Uint128::new(1000),
                    }),
                )
                .unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: Never,
        }];
        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.messages[0], expected_messages[0]);
        assert_eq!(response.events.len(), 1);
        assert_eq!(response.events, expected_events);
    }

    #[test]
    fn test_receive_asset_native() {
        let funds = vec![Coin::new(1000, "uosmo")];
        let info = mock_info("addr", &funds);
        let env = mock_env();
        let asset = Asset::new(AssetInfo::Native("uosmo".into()), 1000u128);
        let result = receive_asset(&info, &env, &asset);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.messages.len(), 0);
        assert_eq!(response.events.len(), 1);
    }

    #[test_case(
        Asset {
            info: AssetInfoBase::Native(String::from("uosmo")),
            amount: Uint128::new(10),
        },vec![Coin::new(10, "uosmo")] => Ok(());
        "Native token received")]
    #[test_case(
        Asset {
            info: AssetInfoBase::Native(String::from("uion")),
            amount: Uint128::new(10),
        },vec![Coin::new(10, "uosmo")] => Err(GenericErr { msg: String::from("Assert native token received failed for asset: uion:10") });
            "Native token not received")]
    #[test_case(
        Asset {
            info: AssetInfoBase::Native(String::from("uosmo")),
            amount: Uint128::new(10),
        },vec![Coin::new(20, "uosmo")] => Err(GenericErr { msg: String::from("Assert native token received failed for asset: uosmo:10") });
                "Native token quantity mismatch")]
    fn test_assert_native_token_received(asset: Asset, funds: Vec<Coin>) -> StdResult<()> {
        let info = MessageInfo {
            funds: funds,
            sender: Addr::unchecked("sender"),
        };
        assert_native_token_received(&info, &asset)
    }

    #[test]
    fn test_separate_natives_and_cw20s() {
        let api = MockApi::default();
        let coins = &vec![
            Coin::new(10, "uosmo"),
            Coin::new(20, "uatom"),
            Coin::new(10, "uion"),
        ];
        let cw20s = &vec![
            Cw20Coin {
                address: "osmo1".to_owned(),
                amount: Uint128::new(100),
            },
            Cw20Coin {
                address: "osmo2".to_owned(),
                amount: Uint128::new(200),
            },
            Cw20Coin {
                address: "osmo3".to_owned(),
                amount: Uint128::new(300),
            },
        ];

        let asset_list = to_asset_list(&api, Some(coins), Some(cw20s)).unwrap();
        let (separated_coins, separated_cw20s) = separate_natives_and_cw20s(&asset_list);

        assert_eq!(separated_coins.len(), coins.len());
        assert_eq!(separated_cw20s.len(), cw20s.len());
        for coin in coins.iter() {
            assert!(separated_coins.contains(coin));
        }
        for cw20 in cw20s.iter() {
            assert!(separated_cw20s.contains(cw20));
        }
    }

    #[test]
    fn test_separate_natives_and_cw20s_with_empty_inputs() {
        let api = MockApi::default();
        let coins = None;
        let cw20s = None;

        let empty_asset_list = to_asset_list(&api, coins, cw20s).unwrap();
        let (coins, cw20s) = separate_natives_and_cw20s(&empty_asset_list);

        assert!(coins.len() == 0);
        assert!(cw20s.len() == 0);
    }

    #[test]
    fn test_to_asset_list() {
        let api = MockApi::default();
        let coins = &vec![
            Coin::new(10, "uosmo"),
            Coin::new(20, "uatom"),
            Coin::new(10, "uion"),
        ];
        let cw20s = &vec![
            Cw20Coin {
                address: "osmo1".to_owned(),
                amount: Uint128::new(100),
            },
            Cw20Coin {
                address: "osmo2".to_owned(),
                amount: Uint128::new(200),
            },
            Cw20Coin {
                address: "osmo3".to_owned(),
                amount: Uint128::new(300),
            },
        ];

        let assets = to_asset_list(&api, Some(coins), Some(cw20s)).unwrap();

        assert_eq!(assets.len(), coins.len() + cw20s.len());
        for coin in coins.iter() {
            assert!(assets
                .find(&AssetInfo::Native(coin.denom.to_owned()))
                .is_some());
        }
        for cw20 in cw20s.iter() {
            assert!(assets
                .find(&AssetInfo::Cw20(api.addr_validate(&cw20.address).unwrap()))
                .is_some());
        }
    }

    #[test]
    fn test_to_asset_list_with_empty_inputs() {
        let api = MockApi::default();
        let coins = None;
        let cw20s = None;

        let assets = to_asset_list(&api, coins, cw20s).unwrap();

        assert!(assets.len() == 0);
    }

    #[test]
    fn test_merge_assets_with_no_duplicates() {
        let mut asset_list = AssetList::new();

        asset_list
            .add(
                &(Asset {
                    info: AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 1"))),
                    amount: Uint128::new(100),
                }),
            )
            .unwrap();
        asset_list
            .add(
                &(Asset {
                    info: AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 2"))),
                    amount: Uint128::new(200),
                }),
            )
            .unwrap();
        asset_list
            .add(
                &(Asset {
                    info: AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 3"))),
                    amount: Uint128::new(300),
                }),
            )
            .unwrap();

        let merged_assets = merge_assets(&asset_list).unwrap();

        assert_eq!(merged_assets.len(), 3);
        assert_eq!(
            merged_assets.to_vec()[0].info,
            AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 1")))
        );
        assert_eq!(merged_assets.to_vec()[0].amount, Uint128::new(100));
        assert_eq!(
            merged_assets.to_vec()[1].info,
            AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 2")))
        );
        assert_eq!(merged_assets.to_vec()[1].amount, Uint128::new(200));
        assert_eq!(
            merged_assets.to_vec()[2].info,
            AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 3")))
        );
        assert_eq!(merged_assets.to_vec()[2].amount, Uint128::new(300));
    }

    #[test]
    fn test_merge_assets_with_duplicates() {
        let mut asset_list = AssetList::new();
        asset_list
            .add(
                &(Asset {
                    info: AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 1"))),
                    amount: Uint128::new(100),
                }),
            )
            .unwrap();
        asset_list
            .add(
                &(Asset {
                    info: AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 1"))),
                    amount: Uint128::new(200),
                }),
            )
            .unwrap();
        let merged_assets = merge_assets(&asset_list).unwrap();

        assert_eq!(merged_assets.len(), 1);
        assert_eq!(
            merged_assets.to_vec()[0].info,
            AssetInfoBase::Cw20(Addr::unchecked(String::from("Asset 1")))
        );
        assert_eq!(merged_assets.to_vec()[0].amount, Uint128::new(300));
    }

    #[test]
    fn test_increase_allowance_msgs() {
        let env = mock_env();
        let spender = Addr::unchecked(String::from("spender"));

        let assets = AssetList::from(vec![
            Asset::new(AssetInfo::Native("uatom".to_string()), Uint128::new(100)),
            Asset::new(
                AssetInfo::Cw20(Addr::unchecked("cw20".to_string())),
                Uint128::new(200),
            ),
        ]);
        let (increase_allowance_msgs, funds) =
            increase_allowance_msgs(&env, &assets, spender.clone()).unwrap();

        assert_eq!(increase_allowance_msgs.len(), 1);
        assert_eq!(
            increase_allowance_msgs[0],
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "cw20".to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                    spender: spender.to_string(),
                    amount: Uint128::new(200),
                    expires: Some(Expiration::AtHeight(env.block.height + 1)),
                })
                .unwrap(),
            })
        );
        assert_eq!(funds.len(), 1);
        assert_eq!(funds[0].amount, Uint128::new(100));
        assert_eq!(funds[0].denom, "uatom");
    }
}

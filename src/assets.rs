use cosmwasm_std::{Api, Coin, Env, MessageInfo, Response, StdError, StdResult};
use cw20::Cw20Coin;
use cw_asset::{Asset, AssetInfo, AssetList};

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
            _ => {}
        }
    }

    (coins, cw20s)
}

/// Assert that a specific native token in the form of an `Asset` was sent to the contract.
pub fn assert_native_token_received(info: &MessageInfo, asset: &Asset) -> StdResult<()> {
    let coin: Coin = asset.try_into()?;

    if !info.funds.contains(&coin) {
        return Err(StdError::generic_err(format!(
            "Assert native token receive failed for asset: {}",
            asset
        )));
    }
    Ok(())
}

/// Calls TransferFrom on an Asset if it is a Cw20. If it is a native we just
/// assert that the native token was already sent to the contract.
///
/// ### Returns
/// Returns a response with the transfer_from message if the asset is a Cw20.
/// Returns an empty response if the asset is a native token.
pub fn receive_asset(info: &MessageInfo, env: &Env, asset: &Asset) -> StdResult<Response> {
    match &asset.info {
        AssetInfo::Cw20(_coin) => {
            let msg =
                asset.transfer_from_msg(info.sender.clone(), env.contract.address.to_string())?;
            Ok(Response::new().add_message(msg))
        }
        AssetInfo::Native(_token) => {
            //Here we just assert that the native token was sent with the contract call
            assert_native_token_received(info, asset)?;
            Ok(Response::new())
        }
        _ => Err(StdError::generic_err("Unsupported asset type")),
    }
}

/// Assert that all assets in the `AssetList` are native tokens.
///
/// ### Returns
/// Returns an error if any of the assets are not native tokens.
/// Returns a `StdResult<Vec<Coin>>` containing the assets as coins if they are all
/// native tokens.
pub fn assert_only_native_coins(assets: AssetList) -> StdResult<Vec<Coin>> {
    assets
        .into_iter()
        .map(assert_native_coin)
        .collect::<StdResult<Vec<Coin>>>()
}

/// Assert that an asset is a native token.
///
/// ### Returns
/// Returns an error if the asset is not a native token.
/// Returns a `StdResult<Coin>` containing the asset as a coin if it is a native token.
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

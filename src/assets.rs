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

use cosmwasm_std::{Api, Coin, StdResult};
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

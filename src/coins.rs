use cosmwasm_std::{Coin, StdError, StdResult, Uint128};
use regex::Regex;

/// Parse coins from string in format {amount}{denom}
pub fn coin_from_str(s: &str) -> Coin {
    // Find index of first non-digit character
    let idx = s
        .char_indices()
        .find(|(_, c)| !c.is_digit(10))
        .map(|(idx, _)| idx)
        .unwrap_or(s.len());

    // Parse amount and denom from string
    let amount: Uint128 = s[..idx].parse::<u128>().unwrap().into();
    let denom = s[idx..].to_string();

    Coin { denom, amount }
}

/// Validate string as a valid CosmosSDK denom according to regex `r"^[a-zA-Z][a-zA-Z0-9/:._-]{2,127}$"`
/// See https://github.com/cosmos/cosmos-sdk/blob/7728516abfab950dc7a9120caad4870f1f962df5/types/coin.go#L865-L867
pub fn validate_string(input: &str) -> StdResult<()> {
    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9/:._-]{2,127}$").unwrap();

    if re.is_match(input) {
        Ok(())
    } else {
        Err(StdError::generic_err(
            "Provided string is not a valid CosmosSDK denom.",
        ))
    }
}

#[test]
fn test_coin_from_sdk_str() {
    let coin = coin_from_str("100000000000000000000gamm/pool/1");

    assert_eq!(coin.amount, Uint128::from(100000000000000000000u128));
    assert_eq!(coin.denom, "gamm/pool/1");
}

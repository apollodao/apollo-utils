use cosmwasm_std::{Coin, Uint128};

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

#[test]
fn test_coin_from_sdk_str() {
    let coin = coin_from_str("100000000000000000000gamm/pool/1");

    assert_eq!(coin.amount, Uint128::from(100000000000000000000u128));
    assert_eq!(coin.denom, "gamm/pool/1");
}

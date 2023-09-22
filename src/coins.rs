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

/// Validate string as a valid CosmosSDK denom according to regex
/// `r"^[a-zA-Z][a-zA-Z0-9/:._-]{2,127}$"`. See https://github.com/cosmos/cosmos-sdk/blob/7728516abfab950dc7a9120caad4870f1f962df5/types/coin.go#L865-L867
pub fn validate_denom(input: &str) -> StdResult<()> {
    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9/:._-]{2,127}$").unwrap();

    if re.is_match(input) {
        Ok(())
    } else {
        Err(StdError::generic_err(
            "Provided string is not a valid CosmosSDK denom.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test]
    fn test_coin_from_sdk_str() {
        let coin = coin_from_str("100000000000000000000gamm/pool/1");

        assert_eq!(coin.amount, Uint128::from(100000000000000000000u128));
        assert_eq!(coin.denom, "gamm/pool/1");
    }

    #[test_case("gamm/pool/1" => Ok(()); "valid osmosis LP denom")]
    #[test_case("uatom" => Ok(()); "valid uatom denom")]
    #[test_case("ibc/C140AFD542AE77BD7DCC83F13FDD8C5E5BB8C4929785E6EC2F4C636F98F17901" => Ok(()); "valid IBC denom")]
    #[test_case("IBC/C140AFD542AE77BD7DCC83F13FDD8C5E5BB8C4929785E6EC2F4C636F98F17901" => Ok(()); "valid IBC denom capital IBC")]
    #[test_case("factory/osmo1g3kmqpp8608szfp0pdag3r6z85npph7wmccat8lgl3mp407kv73qlj7qwp/VaultToken/1/14d/ATOM/OSMO" => Ok(()); "valid token factory denom")]
    #[test_case("test:test/test-test.test_test" => Ok(()); "all valid separators")]
    #[test_case("test test" => Err(StdError::generic_err("Provided string is not a valid CosmosSDK denom.")); "invalid separator space")]
    #[test_case("test/test " => Err(StdError::generic_err("Provided string is not a valid CosmosSDK denom.")); "trailing space")]
    #[test_case(" test/test" => Err(StdError::generic_err("Provided string is not a valid CosmosSDK denom.")); "leading space")]
    #[test_case("/test/test" => Err(StdError::generic_err("Provided string is not a valid CosmosSDK denom.")); "leading separator")]
    #[test_case("2test/test" => Err(StdError::generic_err("Provided string is not a valid CosmosSDK denom.")); "leading number")]
    #[test_case("te" => Err(StdError::generic_err("Provided string is not a valid CosmosSDK denom.")); "too short")]
    #[test_case("tes" => Ok(()); "min length")]
    #[test_case("t//" => Ok(()); "min length with two consecutive separators")]
    #[test_case("testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttest" => Ok(()); "max length")]
    #[test_case("testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttesttestt" => Err(StdError::generic_err("Provided string is not a valid CosmosSDK denom.")); "too long")]
    fn test_validate_denom(input: &str) -> StdResult<()> {
        validate_denom(input)
    }
}

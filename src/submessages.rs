use std::str::FromStr;

use core::fmt::Debug;
use cosmwasm_std::{Event, StdError, StdResult};

/// Parse an attribute string from an [`Event`]
pub fn parse_attribute_value<T: FromStr<Err = E>, E: Debug>(
    event: &Event,
    attr_key: &str,
) -> StdResult<T> {
    T::from_str(
        event
            .attributes
            .iter()
            .find(|attr| attr.key == attr_key)
            .ok_or_else(|| {
                StdError::generic_err(format!(
                    "Event {} event does not contain {} attribute",
                    event.ty, attr_key
                ))
            })?
            .value
            .as_str(),
    )
    .map_err(|e| {
        StdError::generic_err(format!(
            "Failed to parse attribute value from string. Error: {:?}",
            e
        ))
    })
}

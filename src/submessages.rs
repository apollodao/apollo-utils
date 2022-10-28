use std::str::FromStr;

use core::fmt::Debug;
use cosmwasm_std::{Event, StdError, StdResult, SubMsgResponse};

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

/// Find event from SubMsg response
///
/// Returns a [`StdResult`] containing reference to the event if found otherwise [`StdError`]
pub fn find_event<'a>(res: &'a SubMsgResponse, event_type: &str) -> StdResult<&'a Event> {
    res.events
        .iter()
        .find(|event| event.ty == event_type)
        .ok_or(StdError::generic_err(format!(
            "No `{}` event found",
            event_type
        )))
}

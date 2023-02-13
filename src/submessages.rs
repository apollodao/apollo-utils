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
/// Returns a [`StdResult`] containing reference to the event if found otherwise
/// [`StdError`]
pub fn find_event<'a>(res: &'a SubMsgResponse, event_type: &str) -> StdResult<&'a Event> {
    res.events
        .iter()
        .find(|event| event.ty == event_type)
        .ok_or(StdError::generic_err(format!(
            "No `{}` event found",
            event_type
        )))
}

#[cfg(test)]
mod tests {
    use std::num::ParseIntError;

    use cosmwasm_std::Attribute;

    use super::*;

    #[test]
    fn test_find_event_success() {
        let event1 = Event::new("event_type_1");
        let event2 = Event::new("event_type_2");
        let events = vec![event1, event2];
        let res = SubMsgResponse { events, data: None };
        let result = find_event(&res, "event_type_1");
        assert!(result.is_ok());
        let found_event = result.unwrap();
        assert_eq!(found_event.ty, "event_type_1");
    }

    #[test]
    fn test_find_event_not_found() {
        let event1 = Event::new("event_type_1");
        let event2 = Event::new("event_type_2");
        let events = vec![event1, event2];
        let res = SubMsgResponse { events, data: None };
        let result = find_event(&res, "event_type_3");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Generic error: No `event_type_3` event found"
        );
    }

    #[test]
    fn test_parse_attribute_value_success() {
        let attr1 = Attribute::new("key_1", "value_1");
        let attr2 = Attribute::new("key_2", "2");
        let attributes = vec![attr1, attr2];
        let event = Event::new("event_type").add_attributes(attributes);

        let result = parse_attribute_value::<i32, ParseIntError>(&event, "key_2");
        assert!(result.is_ok());
        let parsed_value = result.unwrap();
        assert_eq!(parsed_value, 2);
    }

    #[test]
    fn test_parse_attribute_value_not_found() {
        let attr1 = Attribute::new("key_1", "value_1");
        let attr2 = Attribute::new("key_2", "2");
        let attributes = vec![attr1, attr2];
        let event = Event::new("event_type").add_attributes(attributes);

        let result = parse_attribute_value::<i32, ParseIntError>(&event, "key_3");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Generic error: Event event_type event does not contain key_3 attribute"
        );
    }

    #[test]
    fn test_parse_attribute_value_parse_error() {
        let attr1 = Attribute::new("key_1", "value_1");
        let attr2 = Attribute::new("key_2", "not_a_number");
        let attributes = vec![attr1, attr2];
        let event = Event::new("event_type").add_attributes(attributes);

        let result = parse_attribute_value::<i32, ParseIntError>(&event, "key_2");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Generic error: Failed to parse attribute value from string. Error: ParseIntError { kind: InvalidDigit }"
        );
    }
}

use cosmwasm_std::Response;

/// Merge several Response objects into one. Currently ignores the data fields.
///
/// Returns a new Response object.
pub fn merge_responses(responses: Vec<Response>) -> Response {
    let mut merged = Response::default();
    for response in responses {
        merged = merged
            .add_attributes(response.attributes)
            .add_events(response.events)
            .add_submessages(response.messages);
    }
    merged
}

// Utility macro for response_prefix! macro below
#[macro_export]
macro_rules! with_dollar_sign {
    ( $( $body:tt )* ) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

/// Super-macro for creating the prefix in Events that might differ from contract to contract
/// Simply call `response_prefix!("foo/bar")`, which expands to a definition `response!` with
/// "foo/bar" as the prefix. The subsequent Events will be named "foo/bar/<event name>".
#[macro_export]
macro_rules! response_prefix {
    ( $prefix:literal ) => {
        with_dollar_sign! {
            ( $d:tt ) => {
                /// Macro for generating Response objects with Events, Attributes and Messages
                ///
                /// Variants
                ///  - response!() - returns a new, empty Response
                ///  - response!("event") - returns a new Response with Event named "event"
                ///  - response!(resp, "event") - attaches Event "event" to Response `resp`
                ///  - response!("event", [(k, v), (j, u), ...]) - returns Response with
                ///    Event "event" and Attributes from series (not array) of tuples
                ///  - response!(resp, "event", [(k, v), (j, u), ...]) - same as above but
                ///    attaching to existing Response `resp`
                ///  - response!("event", [(k, v), (j, u), ...], [m, n, ...]) - returns Response
                ///    with Event "event", Attributes `(k, v)` etc., and Messages `m`, `n`, etc.
                ///  - response!(resp, "event", [(k, v), (j, u), ...], [m, n, ...]) - same as
                ///    above but attached to Response `resp`
                macro_rules! response {
                    () => {
                        cosmwasm_std::Response::<cosmwasm_std::Empty>::new()
                    };
                    ( $d response:expr, $d event_name:literal, [ $d( ($d key:literal, $d value:expr) ),* ] ) => {
                        {
                            // needed because things get weird with macros
                            #[allow(unused_mut)]
                            let mut attrs: Vec<cosmwasm_std::Attribute> = Vec::new();
                            $d(
                                attrs.push(cosmwasm_std::attr($d key, $d value));
                            )*
                            let event = cosmwasm_std::Event::new(
                                    format!("{}/{}",
                                    String::from($prefix),
                                    String::from($d event_name)))
                                .add_attributes(attrs.clone());
                            $d response.add_attributes(attrs).add_event(event)
                        }
                    };
                    ( $d response:expr, $d event_name:literal ) => {
                        response!($d response, $d event_name, [])
                    };
                    ( $d event_name:literal ) => {
                        response!(response!(), $d event_name)
                    };
                    ( $d event_name:literal, [ $d( ($d key:literal, $d value:expr) ),* ] ) => {
                        response!(response!(), $d event_name, [ $d(($d key, $d value)),* ])
                    };
                    ( $d response:expr, $d event_name:literal, [ $d( ($d key:literal, $d value:expr) ),* ], [ $d( $d msg:expr ),* ] ) => {
                        {
                            let mut msgs = Vec::new();
                            $d(
                                msgs.push($d msg);
                            )*
                            response!($d response, $d event_name, [ $d(($d key, $d value)),* ]).add_messages(msgs)
                        }
                    };
                    ( $d event_name:literal, [ $d( ($d key:literal, $d value:expr) ),* ], [ $d( $d msg:expr ),* ] ) => {
                        response!(response!(), $d event_name, [ $d(($d key, $d value)),* ], [$d($d msg),*])
                    };
                }
            };
        }
    };
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Attribute, CosmosMsg, Empty, Event, Response};

    response_prefix!("apollo/test");

    #[test]
    fn test_empty_response() {
        let resp = Response::<Empty>::new();
        assert_eq!(resp, response!())
    }

    #[test]
    fn test_response_event() {
        let event = response!("test").events[0].clone();
        assert_eq!(Event::new("apollo/test/test"), event)
    }
}

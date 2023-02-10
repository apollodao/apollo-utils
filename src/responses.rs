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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, BankMsg};

    #[test]
    fn test_merge_empty_responses() {
        let merged = merge_responses(vec![]);

        assert!(merged.attributes.is_empty());
        assert!(merged.events.is_empty());
        assert!(merged.messages.is_empty());
    }

    #[test]
    fn test_merge_responses() {
        let resp1: Response = Response::new()
            .add_attributes(vec![("key1", "value1"), ("send", "1uosmo")])
            .add_messages(vec![BankMsg::Send {
                to_address: String::from("recipient"),
                amount: coins(1, "uosmo"),
            }])
            .set_data(b"data");
        let resp2: Response = Response::new()
            .add_attributes(vec![("key2", "value2"), ("send", "2uosmo")])
            .add_message(BankMsg::Send {
                to_address: String::from("recipient"),
                amount: coins(2, "uosmo"),
            })
            .set_data(b"data2");
        let merged = merge_responses(vec![resp1, resp2]);

        let expected_response: Response = Response::new()
            .add_attributes(vec![
                ("key1", "value1"),
                ("send", "1uosmo"),
                ("key2", "value2"),
                ("send", "2uosmo"),
            ])
            .add_messages(vec![
                BankMsg::Send {
                    to_address: String::from("recipient"),
                    amount: coins(1, "uosmo"),
                },
                BankMsg::Send {
                    to_address: String::from("recipient"),
                    amount: coins(2, "uosmo"),
                },
            ]);
        assert_eq!(merged, expected_response);
        //.set_data(b"data+data2?");
        // TODO: Review this. Data should be None
        assert!(merged.data.is_none());
    }
}

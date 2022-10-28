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

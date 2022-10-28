use cosmwasm_std::{Event, Response, StdError, StdResult, SubMsgResponse};

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

use axum::{body::Bytes, extract::State, http::HeaderMap, response::IntoResponse, Json};
use twilight_model::{
    application::interaction::Interaction, http::interaction::InteractionResponse,
};

use crate::AppState;

// This should support a level slash command as well as a user context menu command for levels
pub async fn handle(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: Bytes,
) -> Result<axum::Json<InteractionResponse>, HandlerError> {
    let body = body.to_vec();
    crate::discord_sig_validation::validate_discord_sig(&headers, &body, &state.pubkey)?;
    // TODO make this actually do something
    let interaction: Interaction = serde_json::from_slice(&body)?;
    Ok(Json(crate::processor::process(interaction, state)?))
}

#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("Signature validation error: {0}")]
    Validation(#[from] crate::discord_sig_validation::SignatureValidationError),
    #[error("serde_json validation error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Command processor error: {0}")]
    CmdProcessor(#[from] crate::processor::CommandProcessorError),
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .body(axum::body::boxed(axum::body::Full::from(self.to_string())))
            .unwrap()
    }
}

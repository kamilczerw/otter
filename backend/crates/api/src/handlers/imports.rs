//! HTTP handler for the CSV transaction import endpoint (`POST
//! /api/v1/imports`): pulls the uploaded file out of a multipart request,
//! runs it through `crate::import::run_import`, and maps the result to an
//! HTTP response per the design doc's "HTTP status codes" section.

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::Json;

use crate::errors::ApiError;
use crate::import::{self, ImportError};
use crate::responses::ImportResponse;

use super::AppState;

/// Multipart field name the uploaded CSV file is expected under.
const FILE_FIELD_NAME: &str = "file";

/// Uploads a bank CSV export and imports it as transactions, auto-creating
/// any months/categories/budget entries they need.
///
/// Accepts a `multipart/form-data` request with the CSV file in a part
/// named `"file"`. The part's `Content-Type` charset parameter (e.g.
/// `charset=windows-1250`) selects the decoder; it defaults to UTF-8 when
/// absent.
///
/// # Errors
///
/// Returns a `400 BAD_REQUEST` [`ApiError`] when the multipart body is
/// malformed, the `"file"` part is missing, the declared charset is
/// unsupported, or the uploaded content isn't structurally valid CSV. A
/// row that fails to extract is not an error here — it is reported in the
/// response body's `failed` list instead.
#[utoipa::path(
    post,
    path = "/api/v1/imports",
    tag = "imports",
    request_body(
        content = String,
        description = "multipart/form-data upload with the CSV file in a part named \"file\"; the part's Content-Type charset (default UTF-8) selects the decoder",
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 201, description = "At least one transaction was newly imported", body = ImportResponse),
        (status = 200, description = "Request succeeded but nothing new was persisted (e.g. a fully-duplicate re-import)", body = ImportResponse),
        (status = 400, description = "Malformed multipart body, missing file part, unsupported charset, or structurally invalid CSV"),
    )
)]
pub async fn import_transactions(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<ImportResponse>), ApiError> {
    let (bytes, content_type) = extract_file_part(multipart).await?;
    let outcome = import::run_import(&state.import_deps, &bytes, content_type.as_deref())
        .await
        .map_err(map_import_error)?;

    let status = if outcome.imported > 0 {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(outcome.into())))
}

/// Reads the `"file"` multipart part's bytes and declared `Content-Type`
/// out of `multipart`.
///
/// # Errors
///
/// Returns a `400 BAD_REQUEST` [`ApiError`] if the multipart body itself is
/// malformed, or if no part named [`FILE_FIELD_NAME`] is present.
async fn extract_file_part(mut multipart: Multipart) -> Result<(Vec<u8>, Option<String>), ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::bad_request(&format!("malformed multipart body: {err}")))?
    {
        if field.name() != Some(FILE_FIELD_NAME) {
            continue;
        }

        let content_type = field.content_type().map(str::to_string);
        let bytes = field
            .bytes()
            .await
            .map_err(|err| ApiError::bad_request(&format!("failed to read uploaded file: {err}")))?;
        return Ok((bytes.to_vec(), content_type));
    }

    Err(ApiError::bad_request(&format!(
        "missing required multipart field \"{FILE_FIELD_NAME}\""
    )))
}

/// Maps an [`ImportError`] (a request-level rejection — unsupported
/// charset, undecodable bytes, or structurally invalid CSV) to a `400`
/// [`ApiError`] with a message identifying which of the three it was.
fn map_import_error(err: ImportError) -> ApiError {
    ApiError::bad_request(&err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `"file"` field name is the documented convention this handler
    /// and its OpenAPI annotation both describe; guard it so a rename isn't
    /// silently inconsistent with the docs above.
    #[test]
    fn file_field_name_is_file() {
        assert_eq!(FILE_FIELD_NAME, "file");
    }
}

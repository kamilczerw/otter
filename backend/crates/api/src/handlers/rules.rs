//! HTTP handlers for the rule-editor capability (`add-rule-editor-ui`):
//! listing the merged default + user rule set, creating/editing/deleting
//! user-stored rules, and importing rules from an uploaded `.toml` file.
//!
//! Config-defined default rules (`AppState::import_deps::inline_rules`) are
//! read-only here — there is no handler path that can reach them by id, so
//! `PUT`/`DELETE` only ever operate on `rule_repo`'s user-stored rules, per
//! the "Editing/deleting a default rule is not possible" requirement.

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;

use domain::entities::MatchingRule;
use domain::errors::MatchingRuleError;
use domain::ports::MatchingRuleRepository;

use crate::errors::ApiError;
use crate::import::{csv_rule_to_new_matching_rule, domain_rule_to_csv_rule};
use crate::requests::MatchingRuleRequest;
use crate::responses::{ImportRulesResponse, MatchingRuleResponse, RuleImportFailureResponse};

use super::{parse_ulid, AppState};

/// Multipart field name the uploaded rules file is expected under, mirroring
/// `handlers::imports`'s `"file"` convention.
const FILE_FIELD_NAME: &str = "file";

/// Lists the complete active rule set: every config-defined default rule (in
/// their config-file order), followed by every user-stored rule (in
/// creation order), per the "List the full rule set in match order"
/// requirement.
///
/// # Errors
///
/// Returns a `500` [`ApiError`] if listing user-stored rules fails.
#[utoipa::path(
    get,
    path = "/api/v1/rules",
    tag = "rules",
    responses(
        (status = 200, description = "Config-defined default rules followed by user-stored rules, in evaluation order", body = Vec<MatchingRuleResponse>),
    )
)]
pub async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<MatchingRuleResponse>>, ApiError> {
    let defaults = state.import_deps.inline_rules.clone();
    let user_rules = state.import_deps.rule_repo.list_all().await?;

    let response = defaults
        .into_iter()
        .map(MatchingRuleResponse::from)
        .chain(user_rules.into_iter().map(MatchingRuleResponse::from))
        .collect();
    Ok(Json(response))
}

/// Creates a new user-stored rule, appended after every existing user rule
/// in evaluation order, per the "Create a user rule" requirement.
///
/// # Errors
///
/// Returns a `400` [`ApiError`] if the request's match criteria are
/// malformed or include an uncompilable regex pattern, or a `500`
/// [`ApiError`] if storage fails.
#[utoipa::path(
    post,
    path = "/api/v1/rules",
    tag = "rules",
    request_body = MatchingRuleRequest,
    responses(
        (status = 201, description = "The rule was created", body = MatchingRuleResponse),
        (status = 400, description = "Malformed match criteria or an uncompilable regex pattern"),
    )
)]
pub async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<MatchingRuleRequest>,
) -> Result<(StatusCode, Json<MatchingRuleResponse>), ApiError> {
    let new_rule = req.try_into()?;
    let rule = state.import_deps.rule_repo.create(new_rule).await?;
    Ok((StatusCode::CREATED, Json(rule.into())))
}

/// Replaces an existing user rule's match criteria and category in place,
/// without changing its position in the evaluation order, per the "Edit a
/// user rule in place" requirement.
///
/// # Errors
///
/// Returns a `400` [`ApiError`] for malformed/invalid match criteria, a
/// `404` [`ApiError`] if `id` isn't a known user rule (a default rule's id
/// never matches, since defaults have none), or a `500` [`ApiError`] if
/// storage fails.
#[utoipa::path(
    put,
    path = "/api/v1/rules/{id}",
    tag = "rules",
    params(("id" = String, Path, description = "User rule ULID")),
    request_body = MatchingRuleRequest,
    responses(
        (status = 200, description = "The rule was updated", body = MatchingRuleResponse),
        (status = 400, description = "Malformed match criteria or an uncompilable regex pattern"),
        (status = 404, description = "No user rule exists with this id"),
    )
)]
pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MatchingRuleRequest>,
) -> Result<Json<MatchingRuleResponse>, ApiError> {
    let ulid = parse_ulid(&id)?;
    let new_rule = req.try_into()?;
    let rule = state.import_deps.rule_repo.update(&ulid, new_rule).await?;
    Ok(Json(rule.into()))
}

/// Permanently removes a user-stored rule, per the "Delete a user rule"
/// requirement.
///
/// # Errors
///
/// Returns a `404` [`ApiError`] if `id` isn't a known user rule, or a `500`
/// [`ApiError`] if storage fails.
#[utoipa::path(
    delete,
    path = "/api/v1/rules/{id}",
    tag = "rules",
    params(("id" = String, Path, description = "User rule ULID")),
    responses(
        (status = 204, description = "The rule was deleted"),
        (status = 404, description = "No user rule exists with this id"),
    )
)]
pub async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let ulid = parse_ulid(&id)?;
    state.import_deps.rule_repo.delete(&ulid).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Uploads a `.toml` rules file (the same hand-editable shape produced by
/// `csv_import::save_rules`) and adds its rules to the user's stored rule
/// set, skipping exact duplicates of an existing user rule and collecting
/// per-rule regex-compile failures without aborting the rest of the batch,
/// per the "Import rules from a file" requirement.
///
/// # Errors
///
/// Returns a `400` [`ApiError`] if the multipart body is malformed, the
/// `"file"` part is missing, or the uploaded content isn't valid TOML
/// matching the rules-file shape. A rule whose pattern fails to compile is
/// not an error here — it is reported in the response body's `failed` list
/// instead.
#[utoipa::path(
    post,
    path = "/api/v1/rules/import",
    tag = "rules",
    request_body(
        content = String,
        description = "multipart/form-data upload with the .toml rules file in a part named \"file\"",
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 201, description = "The upload was processed (imported/skipped/failed counts reported)", body = ImportRulesResponse),
        (status = 400, description = "Malformed multipart body, missing file part, or invalid TOML"),
    )
)]
pub async fn import_rules(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<ImportRulesResponse>), ApiError> {
    let bytes = extract_rules_file_part(multipart).await?;
    let uploaded = parse_rules_file(&bytes)?;

    let existing_user_rules = state.import_deps.rule_repo.list_all().await?;
    let outcome =
        import_new_rules(state.import_deps.rule_repo.as_ref(), uploaded, &existing_user_rules).await?;

    Ok((StatusCode::CREATED, Json(outcome)))
}

/// Reads the `"file"` multipart part's bytes out of `multipart`.
///
/// # Errors
///
/// Returns a `400 BAD_REQUEST` [`ApiError`] if the multipart body itself is
/// malformed, or if no part named [`FILE_FIELD_NAME`] is present.
async fn extract_rules_file_part(mut multipart: Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::bad_request(&format!("malformed multipart body: {err}")))?
    {
        if field.name() != Some(FILE_FIELD_NAME) {
            continue;
        }

        let bytes = field
            .bytes()
            .await
            .map_err(|err| ApiError::bad_request(&format!("failed to read uploaded file: {err}")))?;
        return Ok(bytes.to_vec());
    }

    Err(ApiError::bad_request(&format!(
        "missing required multipart field \"{FILE_FIELD_NAME}\""
    )))
}

/// Parses uploaded bytes as UTF-8 TOML matching `csv_import::RulesFile`'s
/// shape, returning its rule list.
///
/// # Errors
///
/// Returns a `400 BAD_REQUEST` [`ApiError`] if the bytes aren't valid UTF-8
/// or don't parse as a [`csv_import::RulesFile`].
fn parse_rules_file(bytes: &[u8]) -> Result<Vec<csv_import::Rule>, ApiError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|err| ApiError::bad_request(&format!("uploaded file isn't valid UTF-8: {err}")))?;
    let rules_file: csv_import::RulesFile = toml::from_str(text)
        .map_err(|err| ApiError::bad_request(&format!("failed to parse rules file as TOML: {err}")))?;
    Ok(rules_file.matching_rules)
}

/// Stores every `uploaded` rule that isn't an exact structural duplicate
/// (same criteria and category, see the design doc's "Dedup is exact
/// structural equality" decision) of one of `existing_user_rules`,
/// collecting per-rule regex-compile failures instead of aborting the batch.
///
/// # Errors
///
/// Returns a `500` [`ApiError`] if storing a non-duplicate, validly
/// constructed rule fails.
async fn import_new_rules(
    rule_repo: &(dyn MatchingRuleRepository + '_),
    uploaded: Vec<csv_import::Rule>,
    existing_user_rules: &[MatchingRule],
) -> Result<ImportRulesResponse, ApiError> {
    let existing_csv_rules: Vec<csv_import::Rule> = existing_user_rules
        .iter()
        .cloned()
        .map(domain_rule_to_csv_rule)
        .collect();

    let mut imported = 0;
    let mut skipped = 0;
    let mut failed = Vec::new();

    for rule in uploaded {
        if existing_csv_rules.contains(&rule) {
            skipped += 1;
            continue;
        }

        let category = rule.category.clone();
        match csv_rule_to_new_matching_rule(rule) {
            Ok(new_rule) => {
                rule_repo.create(new_rule).await?;
                imported += 1;
            }
            Err(MatchingRuleError::InvalidRegex { pattern, reason }) => {
                failed.push(RuleImportFailureResponse {
                    category,
                    reason: format!("invalid regex pattern {pattern:?}: {reason}"),
                });
            }
            Err(other) => return Err(other.into()),
        }
    }

    Ok(ImportRulesResponse {
        imported,
        skipped,
        failed,
    })
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

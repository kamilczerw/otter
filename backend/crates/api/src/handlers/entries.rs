use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use domain::errors::EntryError;
use domain::types::{DueDay, Money};

use crate::errors::ApiError;
use crate::requests::{CreateEntryRequest, UpdateEntryRequest};
use crate::responses::EntryResponse;

use super::{parse_ulid, AppState};

pub async fn list_entries(
    State(state): State<AppState>,
    Path(month_id): Path<String>,
) -> Result<Json<Vec<EntryResponse>>, ApiError> {
    let month_ulid = parse_ulid(&month_id)?;
    let entries = state.entry_service.list_by_month(&month_ulid).await?;
    let response: Vec<EntryResponse> = entries.into_iter().map(|e| e.into()).collect();
    Ok(Json(response))
}

pub async fn create_entry(
    State(state): State<AppState>,
    Path(month_id): Path<String>,
    Json(req): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<EntryResponse>), ApiError> {
    let month_ulid = parse_ulid(&month_id)?;
    let category_ulid = parse_ulid(&req.category_id)?;

    let due_day = match req.due_day {
        Some(d) => {
            let dd = DueDay::new(d).map_err(|_| EntryError::InvalidDueDay { value: d })?;
            Some(dd)
        }
        None => None,
    };

    let budgeted = Money::new(req.budgeted);

    let entry = state
        .entry_service
        .create(month_ulid, category_ulid, budgeted, due_day)
        .await?;
    Ok((StatusCode::CREATED, Json(entry.into())))
}

pub async fn update_entry(
    State(state): State<AppState>,
    Path((_month_id, entry_id)): Path<(String, String)>,
    Json(req): Json<UpdateEntryRequest>,
) -> Result<Json<EntryResponse>, ApiError> {
    let entry_ulid = parse_ulid(&entry_id)?;

    let budgeted = req.budgeted.map(Money::new);

    let due_day = match req.due_day {
        Some(None) => Some(None),
        Some(Some(d)) => {
            let dd = DueDay::new(d).map_err(|_| EntryError::InvalidDueDay { value: d })?;
            Some(Some(dd))
        }
        None => None,
    };

    let entry = state
        .entry_service
        .update(&entry_ulid, budgeted, due_day)
        .await?;
    Ok(Json(entry.into()))
}

pub async fn delete_entry(
    State(state): State<AppState>,
    Path((_month_id, entry_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let entry_ulid = parse_ulid(&entry_id)?;
    state.entry_service.delete(&entry_ulid).await?;
    Ok(StatusCode::NO_CONTENT)
}

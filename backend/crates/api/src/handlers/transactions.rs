use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use domain::errors::TransactionError;
use domain::types::{Money, TransactionDate};

use crate::errors::ApiError;
use crate::requests::{CreateTransactionRequest, TransactionListQuery, UpdateTransactionRequest};
use crate::responses::{PaginatedTransactionsResponse, TransactionResponse};

use super::{parse_ulid, AppState};

pub async fn list_transactions(
    State(state): State<AppState>,
    Query(query): Query<TransactionListQuery>,
) -> Result<Response, ApiError> {
    if let Some(ref entry_id_str) = query.entry_id {
        // Per-entry paginated mode
        let entry_ulid = parse_ulid(entry_id_str)?;
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        // Fetch limit + 1 to determine has_more
        let transactions = state
            .transaction_service
            .list_by_entry(&entry_ulid, limit + 1, offset)
            .await?;

        let has_more = transactions.len() > limit as usize;
        let items: Vec<TransactionResponse> = transactions
            .into_iter()
            .take(limit as usize)
            .map(|t| t.into())
            .collect();

        let response = PaginatedTransactionsResponse { items, has_more };
        Ok(Json(response).into_response())
    } else {
        // Legacy month-based mode
        let month_str = match query.month {
            Some(m) => m,
            None => return Err(ApiError::month_required()),
        };
        let month_ulid = parse_ulid(&month_str)?;
        let transactions = state
            .transaction_service
            .list_by_month(&month_ulid)
            .await?;
        let response: Vec<TransactionResponse> = transactions.into_iter().map(|t| t.into()).collect();
        Ok(Json(response).into_response())
    }
}

pub async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), ApiError> {
    let entry_ulid = parse_ulid(&req.entry_id)?;
    let date: TransactionDate = req.date.parse().map_err(|_| TransactionError::InvalidDate {
        value: req.date.clone(),
    })?;
    let amount = Money::new(req.amount);

    let transaction = state
        .transaction_service
        .create(entry_ulid, amount, date, req.title)
        .await?;
    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub async fn update_transaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTransactionRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let ulid = parse_ulid(&id)?;

    let entry_id = match req.entry_id {
        Some(ref eid) => Some(parse_ulid(eid)?),
        None => None,
    };

    let amount = req.amount.map(Money::new);

    let date = match req.date {
        Some(ref d) => {
            let td: TransactionDate =
                d.parse()
                    .map_err(|_| TransactionError::InvalidDate { value: d.clone() })?;
            Some(td)
        }
        None => None,
    };

    let transaction = state
        .transaction_service
        .update(&ulid, entry_id, amount, date, req.title)
        .await?;
    Ok(Json(transaction.into()))
}

pub async fn delete_transaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let ulid = parse_ulid(&id)?;
    state.transaction_service.delete(&ulid).await?;
    Ok(StatusCode::NO_CONTENT)
}

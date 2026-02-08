use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use domain::errors::MonthError;
use domain::types::BudgetMonth;

use crate::errors::ApiError;
use crate::requests::CreateMonthRequest;
use crate::responses::MonthResponse;

use super::{parse_ulid, AppState};

pub async fn list_months(
    State(state): State<AppState>,
) -> Result<Json<Vec<MonthResponse>>, ApiError> {
    let months = state.month_service.list_all().await?;
    let response: Vec<MonthResponse> = months.into_iter().map(|m| m.into()).collect();
    Ok(Json(response))
}

pub async fn get_month(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MonthResponse>, ApiError> {
    let ulid = parse_ulid(&id)?;
    let month = state.month_service.find_by_id(&ulid).await?;
    Ok(Json(month.into()))
}

pub async fn create_month(
    State(state): State<AppState>,
    Json(req): Json<CreateMonthRequest>,
) -> Result<(StatusCode, Json<MonthResponse>), ApiError> {
    let budget_month: BudgetMonth = req.month.parse().map_err(|_| MonthError::InvalidFormat {
        value: req.month.clone(),
    })?;
    let month = state.month_service.create(budget_month).await?;
    Ok((StatusCode::CREATED, Json(month.into())))
}

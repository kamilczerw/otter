use axum::extract::{Path, State};
use axum::Json;

use crate::errors::ApiError;
use crate::responses::MonthSummaryResponse;

use super::{parse_ulid, AppState};

pub async fn get_month_summary(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MonthSummaryResponse>, ApiError> {
    let ulid = parse_ulid(&id)?;
    let summary = state.summary_service.get_month_summary(&ulid).await?;
    Ok(Json(summary.into()))
}

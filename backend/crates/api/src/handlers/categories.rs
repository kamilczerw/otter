use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use domain::errors::CategoryError;
use domain::types::CategoryName;

use crate::errors::ApiError;
use crate::requests::CreateCategoryRequest;
use crate::requests::UpdateCategoryRequest;
use crate::responses::CategoryResponse;

use super::{parse_ulid, AppState};

pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let categories = state.category_service.list_all().await?;
    let response: Vec<CategoryResponse> = categories.into_iter().map(|c| c.into()).collect();
    Ok(Json(response))
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), ApiError> {
    let name = CategoryName::new(req.name).map_err(|e| {
        CategoryError::InvalidNameFormat {
            reason: e.to_string(),
        }
    })?;
    let category = state.category_service.create(name).await?;
    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let ulid = parse_ulid(&id)?;
    let name = CategoryName::new(req.name).map_err(|e| {
        CategoryError::InvalidNameFormat {
            reason: e.to_string(),
        }
    })?;
    let category = state.category_service.rename(&ulid, name).await?;
    Ok(Json(category.into()))
}

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use domain::errors::{CategoryError, EntryError, MonthError, TransactionError};

pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub details: Option<Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = if let Some(details) = self.details {
            json!({ "error": { "code": self.code, "details": details } })
        } else {
            json!({ "error": { "code": self.code } })
        };
        (self.status, Json(body)).into_response()
    }
}

impl ApiError {
    pub fn bad_request(reason: &str) -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST".into(),
            details: Some(json!({ "reason": reason })),
        }
    }

    pub fn month_required() -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "TRANSACTIONS_MONTH_REQUIRED".into(),
            details: None,
        }
    }
}

impl From<CategoryError> for ApiError {
    fn from(err: CategoryError) -> Self {
        match err {
            CategoryError::NotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "CATEGORY_NOT_FOUND".into(),
                details: None,
            },
            CategoryError::NameAlreadyExists { name } => ApiError {
                status: StatusCode::CONFLICT,
                code: "CATEGORY_NAME_ALREADY_EXISTS".into(),
                details: Some(json!({ "name": name })),
            },
            CategoryError::InvalidNameFormat { reason } => ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "CATEGORY_INVALID_NAME".into(),
                details: Some(json!({ "reason": reason })),
            },
            CategoryError::Repository(msg) => {
                tracing::error!("Category repository error: {}", msg);
                ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR".into(),
                    details: None,
                }
            }
        }
    }
}

impl From<MonthError> for ApiError {
    fn from(err: MonthError) -> Self {
        match err {
            MonthError::NotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "MONTH_NOT_FOUND".into(),
                details: None,
            },
            MonthError::AlreadyExists { month } => ApiError {
                status: StatusCode::CONFLICT,
                code: "MONTH_ALREADY_EXISTS".into(),
                details: Some(json!({ "month": month })),
            },
            MonthError::InvalidFormat { value } => ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "MONTH_INVALID_FORMAT".into(),
                details: Some(json!({ "value": value })),
            },
            MonthError::NoSourceMonthForCopy => {
                tracing::error!("No source month for copy");
                ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR".into(),
                    details: None,
                }
            }
            MonthError::Repository(msg) => {
                tracing::error!("Month repository error: {}", msg);
                ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR".into(),
                    details: None,
                }
            }
        }
    }
}

impl From<EntryError> for ApiError {
    fn from(err: EntryError) -> Self {
        match err {
            EntryError::NotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "ENTRY_NOT_FOUND".into(),
                details: None,
            },
            EntryError::CategoryAlreadyInMonth { category_id, month } => ApiError {
                status: StatusCode::CONFLICT,
                code: "ENTRY_CATEGORY_ALREADY_IN_MONTH".into(),
                details: Some(json!({ "category_id": category_id, "month": month })),
            },
            EntryError::HasTransactions { transaction_count } => ApiError {
                status: StatusCode::CONFLICT,
                code: "ENTRY_HAS_TRANSACTIONS".into(),
                details: Some(json!({ "transaction_count": transaction_count })),
            },
            EntryError::InvalidDueDay { value } => ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "ENTRY_INVALID_DUE_DAY".into(),
                details: Some(json!({ "value": value })),
            },
            EntryError::CategoryNotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "CATEGORY_NOT_FOUND".into(),
                details: None,
            },
            EntryError::MonthNotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "MONTH_NOT_FOUND".into(),
                details: None,
            },
            EntryError::Repository(msg) => {
                tracing::error!("Entry repository error: {}", msg);
                ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR".into(),
                    details: None,
                }
            }
        }
    }
}

impl From<TransactionError> for ApiError {
    fn from(err: TransactionError) -> Self {
        match err {
            TransactionError::NotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "TRANSACTION_NOT_FOUND".into(),
                details: None,
            },
            TransactionError::EntryNotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "TRANSACTION_ENTRY_NOT_FOUND".into(),
                details: None,
            },
            TransactionError::InvalidAmount { value } => ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "TRANSACTION_INVALID_AMOUNT".into(),
                details: Some(json!({ "value": value })),
            },
            TransactionError::InvalidDate { value } => ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "TRANSACTION_INVALID_DATE".into(),
                details: Some(json!({ "value": value })),
            },
            TransactionError::Repository(msg) => {
                tracing::error!("Transaction repository error: {}", msg);
                ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR".into(),
                    details: None,
                }
            }
        }
    }
}

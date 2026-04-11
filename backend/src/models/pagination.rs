#[derive(utoipa::ToSchema)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub offset: i64,
}

impl Pagination {
    pub fn new(
        page: Option<i64>,
        per_page: Option<i64>,
        default_per_page: i64,
        max_per_page: i64,
    ) -> Self {
        let page = page.unwrap_or(1).max(1);
        let per_page = per_page
            .unwrap_or(default_per_page)
            .min(max_per_page)
            .max(1);
        let offset = (page - 1) * per_page;
        Self {
            page,
            per_page,
            offset,
        }
    }

    pub fn from_filter(page: Option<i64>, per_page: Option<i64>) -> Self {
        Self::new(page, per_page, 50, 200)
    }
}

use axum::Json;
use serde_json::{Value, json};

use crate::errors::AppError;

pub async fn paginated<L, LF, C, CF, T>(
    page: Option<i64>,
    per_page: Option<i64>,
    list_fn: L,
    count_fn: C,
) -> Result<Json<Value>, AppError>
where
    LF: std::future::Future<Output = Result<Vec<T>, AppError>>,
    CF: std::future::Future<Output = Result<i64, AppError>>,
    T: serde::Serialize,
    L: FnOnce() -> LF,
    C: FnOnce() -> CF,
{
    let pag = Pagination::from_filter(page, per_page);
    let (data, total) = tokio::join!(list_fn(), count_fn());
    Ok(Json(json!({
        "data": data?,
        "total": total?,
        "page": pag.page,
        "per_page": pag.per_page,
    })))
}

pub async fn simple_list<F, C, T>(
    list_fn: F,
    count_fn: C,
) -> Result<Json<Value>, AppError>
where
    F: std::future::Future<Output = Result<Vec<T>, AppError>>,
    C: std::future::Future<Output = Result<i64, AppError>>,
    T: serde::Serialize,
{
    let (data, total) = tokio::join!(list_fn, count_fn);
    let data = data?;
    let total = total?;
    Ok(Json(json!({
        "data": data,
        "total": total,
        "page": 1,
        "per_page": total,
    })))
}

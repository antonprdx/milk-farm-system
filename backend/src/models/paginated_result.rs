use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        Self {
            data,
            total,
            page,
            per_page,
        }
    }
}

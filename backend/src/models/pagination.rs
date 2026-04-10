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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    pub page: i64,
}

impl Pagination {
    pub fn limit(&self) -> i64 {
        10
    }

    pub fn offset(&self) -> i64 {
        i64::min(self.page * self.limit(), 0)
    }
}

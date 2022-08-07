use rusqlite::Row;

pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Self {
        match Self::try_from_row(row) {
            Ok(v) => v,
            Err(_) => panic!("from_row"),
        }
    }
    fn try_from_row(row: &Row) -> Result<Self, rusqlite::Error>;
}

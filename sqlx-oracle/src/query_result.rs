use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct OracleQueryResult {
    pub rows_affected: u64,
}

impl OracleQueryResult {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<OracleQueryResult> for OracleQueryResult {
    fn extend<T: IntoIterator<Item = OracleQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}

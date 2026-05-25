use std::iter::{Extend, IntoIterator};

/// Oracle 查询结果（DML）。
///
/// 记录受影响的记录行数。
#[derive(Debug, Default)]
pub struct OracleQueryResult {
    pub rows_affected: u64,
}

impl OracleQueryResult {
    /// 返回 DML 语句影响的行数。
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<OracleQueryResult> for OracleQueryResult {
    /// 累加多个查询结果的影响行数。
    fn extend<T: IntoIterator<Item = OracleQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // 基本构造与默认值
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_zero() {
        let r = OracleQueryResult::default();
        assert_eq!(r.rows_affected(), 0);
    }

    #[test]
    fn test_rows_affected() {
        let r = OracleQueryResult { rows_affected: 42 };
        assert_eq!(r.rows_affected(), 42);
    }

    // -----------------------------------------------------------------------
    // Extend 累加逻辑
    // -----------------------------------------------------------------------

    #[test]
    fn test_extend_single() {
        let mut r = OracleQueryResult { rows_affected: 10 };
        r.extend([OracleQueryResult { rows_affected: 5 }]);
        assert_eq!(r.rows_affected(), 15);
    }

    #[test]
    fn test_extend_multiple() {
        let mut r = OracleQueryResult { rows_affected: 0 };
        r.extend([
            OracleQueryResult { rows_affected: 1 },
            OracleQueryResult { rows_affected: 2 },
            OracleQueryResult { rows_affected: 3 },
        ]);
        assert_eq!(r.rows_affected(), 6);
    }

    #[test]
    fn test_extend_empty() {
        let mut r = OracleQueryResult { rows_affected: 7 };
        r.extend([]);
        assert_eq!(r.rows_affected(), 7);
    }

    #[test]
    fn test_extend_large() {
        let mut r = OracleQueryResult { rows_affected: 0 };
        let many: Vec<_> = (0..100).map(|i| OracleQueryResult { rows_affected: i }).collect();
        r.extend(many);
        // 0..100 的和 = 4950
        assert_eq!(r.rows_affected(), 4950);
    }

    // -----------------------------------------------------------------------
    // Debug / Default trait
    // -----------------------------------------------------------------------

    #[test]
    fn test_debug() {
        let r = OracleQueryResult { rows_affected: 99 };
        let debug = format!("{r:?}");
        assert!(debug.contains("99"));
    }

    #[test]
    fn test_default_trait() {
        let r: OracleQueryResult = Default::default();
        assert_eq!(r.rows_affected(), 0);
    }
}

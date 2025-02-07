use std::fmt;

use crate::store::postgres::query::{AliasedColumn, Expression, Transpile};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct WindowStatement<'q> {
    partition: Vec<Expression<'q>>,
}

impl<'q> WindowStatement<'q> {
    pub fn partition_by(column: AliasedColumn<'q>) -> Self {
        Self {
            partition: vec![Expression::Column(column)],
        }
    }
}

impl Transpile for WindowStatement<'_> {
    fn transpile(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("PARTITION BY ")?;
        for (idx, partition) in self.partition.iter().enumerate() {
            partition.transpile(fmt)?;
            if idx + 1 < self.partition.len() {
                fmt.write_str(", ")?;
            }
        }

        Ok(())
    }
}

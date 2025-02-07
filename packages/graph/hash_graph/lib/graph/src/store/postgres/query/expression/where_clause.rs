use std::fmt;

use crate::store::postgres::query::{Condition, Transpile};

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct WhereExpression<'q> {
    conditions: Vec<Condition<'q>>,
}

impl<'q> WhereExpression<'q> {
    pub fn add_condition(&mut self, condition: Condition<'q>) {
        self.conditions.push(condition);
    }

    pub fn len(&self) -> usize {
        self.conditions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

impl Transpile for WhereExpression<'_> {
    fn transpile(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.conditions.is_empty() {
            return Ok(());
        }

        fmt.write_str("WHERE ")?;
        for (idx, condition) in self.conditions.iter().enumerate() {
            if idx > 0 {
                fmt.write_str(" AND ")?;
            }
            condition.transpile(fmt)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use type_system::DataType;

    use super::*;
    use crate::{
        ontology::DataTypeQueryPath,
        store::{
            postgres::query::{test_helper::trim_whitespace, SelectCompiler},
            query::{Filter, FilterExpression, Parameter},
        },
    };

    #[test]
    fn transpile_where_expression() {
        let mut compiler = SelectCompiler::<DataType>::new();
        let mut where_clause = WhereExpression::default();
        assert_eq!(where_clause.transpile_to_string(), "");

        let filter_a = Filter::<DataType>::Equal(
            Some(FilterExpression::Path(DataTypeQueryPath::Version)),
            Some(FilterExpression::Parameter(Parameter::Text(Cow::Borrowed(
                "latest",
            )))),
        );
        where_clause.add_condition(compiler.compile_filter(&filter_a));

        assert_eq!(
            where_clause.transpile_to_string(),
            r#"WHERE "type_ids_0_1_0"."version" = "type_ids_0_1_0"."latest_version""#
        );

        let filter_b = Filter::<DataType>::All(vec![
            Filter::Equal(
                Some(FilterExpression::Path(DataTypeQueryPath::BaseUri)),
                Some(FilterExpression::Parameter(Parameter::Text(Cow::Borrowed(
                    "https://blockprotocol.org/@blockprotocol/types/data-type/text/",
                )))),
            ),
            Filter::Equal(
                Some(FilterExpression::Path(DataTypeQueryPath::Version)),
                Some(FilterExpression::Parameter(Parameter::Number(1.0))),
            ),
        ]);
        where_clause.add_condition(compiler.compile_filter(&filter_b));

        assert_eq!(
            trim_whitespace(where_clause.transpile_to_string()),
            trim_whitespace(
                r#"
                WHERE "type_ids_0_1_0"."version" = "type_ids_0_1_0"."latest_version"
                  AND ("type_ids_0_1_0"."base_uri" = $1) AND ("type_ids_0_1_0"."version" = $2)"#
            )
        );

        let filter_c = Filter::<DataType>::NotEqual(
            Some(FilterExpression::Path(DataTypeQueryPath::Description)),
            None,
        );
        where_clause.add_condition(compiler.compile_filter(&filter_c));

        assert_eq!(
            trim_whitespace(where_clause.transpile_to_string()),
            trim_whitespace(
                r#"
                WHERE "type_ids_0_1_0"."version" = "type_ids_0_1_0"."latest_version"
                  AND ("type_ids_0_1_0"."base_uri" = $1) AND ("type_ids_0_1_0"."version" = $2)
                  AND "data_types_0_0_0"."schema"->>'description' IS NOT NULL"#
            )
        );

        let filter_d = Filter::<DataType>::Any(vec![
            Filter::Equal(
                Some(FilterExpression::Path(DataTypeQueryPath::Title)),
                Some(FilterExpression::Parameter(Parameter::Text(Cow::Borrowed(
                    "some title",
                )))),
            ),
            Filter::Equal(
                Some(FilterExpression::Path(DataTypeQueryPath::Description)),
                Some(FilterExpression::Parameter(Parameter::Text(Cow::Borrowed(
                    "some description",
                )))),
            ),
        ]);
        where_clause.add_condition(compiler.compile_filter(&filter_d));

        assert_eq!(
            trim_whitespace(where_clause.transpile_to_string()),
            trim_whitespace(
                r#"
                WHERE "type_ids_0_1_0"."version" = "type_ids_0_1_0"."latest_version"
                  AND ("type_ids_0_1_0"."base_uri" = $1) AND ("type_ids_0_1_0"."version" = $2)
                  AND "data_types_0_0_0"."schema"->>'description' IS NOT NULL
                  AND (("data_types_0_0_0"."schema"->>'title' = $3) OR ("data_types_0_0_0"."schema"->>'description' = $4))"#
            )
        );

        let parameters = compiler
            .compile()
            .1
            .iter()
            .map(|parameter| format!("{parameter:?}"))
            .collect::<Vec<_>>();
        assert_eq!(parameters, &[
            "\"https://blockprotocol.org/@blockprotocol/types/data-type/text/\"",
            "1.0",
            "\"some title\"",
            "\"some description\""
        ]);
    }
}

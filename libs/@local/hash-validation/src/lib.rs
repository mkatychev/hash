#![feature(lint_reasons)]
#![expect(
    clippy::missing_errors_doc,
    reason = "It's obvious that validation may error on invalid data."
)]

pub mod error;

pub use self::{
    data_type::{DataTypeConstraint, DataValidationError, JsonSchemaValueType},
    entity_type::EntityValidationError,
    property_type::PropertyValidationError,
};

mod data_type;
mod entity_type;
mod property_type;

use std::{borrow::Borrow, future::Future};

use error_stack::{Context, Report};
use type_system::url::VersionedUrl;

trait Schema<V: ?Sized, P: Sync> {
    type Error: Context;

    fn validate_value<'a>(
        &'a self,
        value: &'a V,
        provider: &'a P,
    ) -> impl Future<Output = Result<(), Report<Self::Error>>> + Send + 'a;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Valid<T> {
    value: T,
}

impl<T> Valid<T> {
    pub async fn new<S, C>(value: T, schema: S, context: C) -> Result<Self, Report<T::Error>>
    where
        T: Validate<S, C> + Send,
        S: Send,
        C: Send,
    {
        value.validate(&schema, &context).await?;
        Ok(Self { value })
    }

    pub fn into_unvalidated(self) -> T {
        self.value
    }
}

impl<T> AsRef<T> for Valid<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> Borrow<T> for Valid<T> {
    fn borrow(&self) -> &T {
        &self.value
    }
}

pub trait Validate<S, C> {
    type Error: Context;

    fn validate(
        &self,
        schema: &S,
        context: &C,
    ) -> impl Future<Output = Result<(), Report<Self::Error>>> + Send;
}

pub trait OntologyTypeProvider<O> {
    fn provide_type(
        &self,
        type_id: &VersionedUrl,
    ) -> impl Future<Output = Result<impl Borrow<O> + Send, Report<impl Context>>> + Send;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use graph_types::knowledge::entity::EntityProperties;
    use serde_json::Value as JsonValue;
    use thiserror::Error;
    use type_system::{raw, DataType, EntityType, PropertyType};

    use super::*;
    use crate::{
        data_type::DataValidationError, entity_type::EntityValidationError,
        error::install_error_stack_hooks, property_type::PropertyValidationError,
    };

    struct Provider {
        property_types: HashMap<VersionedUrl, PropertyType>,
        data_types: HashMap<VersionedUrl, DataType>,
    }
    impl Provider {
        fn new(
            property_types: impl IntoIterator<Item = PropertyType>,
            data_types: impl IntoIterator<Item = DataType>,
        ) -> Self {
            Self {
                property_types: property_types
                    .into_iter()
                    .map(|schema| (schema.id().clone(), schema))
                    .collect(),
                data_types: data_types
                    .into_iter()
                    .map(|schema| (schema.id().clone(), schema))
                    .collect(),
            }
        }
    }

    #[derive(Debug, Error)]
    #[error("property type was not found: `{id}`")]
    struct InvalidPropertyType {
        id: VersionedUrl,
    }
    #[derive(Debug, Error)]
    #[error("data type was not found: `{id}`")]
    struct InvalidDataType {
        id: VersionedUrl,
    }

    impl OntologyTypeProvider<PropertyType> for Provider {
        async fn provide_type(
            &self,
            type_id: &VersionedUrl,
        ) -> Result<&PropertyType, Report<InvalidPropertyType>> {
            self.property_types.get(type_id).ok_or_else(|| {
                Report::new(InvalidPropertyType {
                    id: type_id.clone(),
                })
            })
        }
    }

    impl OntologyTypeProvider<DataType> for Provider {
        async fn provide_type(
            &self,
            type_id: &VersionedUrl,
        ) -> Result<&DataType, Report<InvalidDataType>> {
            self.data_types.get(type_id).ok_or_else(|| {
                Report::new(InvalidDataType {
                    id: type_id.clone(),
                })
            })
        }
    }

    pub(crate) async fn validate_entity(
        entity: &'static str,
        entity_type: &'static str,
        property_types: impl IntoIterator<Item = &'static str> + Send,
        data_types: impl IntoIterator<Item = &'static str> + Send,
    ) -> Result<(), Report<EntityValidationError>> {
        install_error_stack_hooks();

        let provider = Provider::new(
            property_types.into_iter().map(|property_type_id| {
                let raw_property_type = serde_json::from_str::<raw::PropertyType>(property_type_id)
                    .expect("failed to read property type string");
                PropertyType::try_from(raw_property_type).expect("failed to parse property type")
            }),
            data_types.into_iter().map(|data_type_id| {
                let raw_data_type = serde_json::from_str::<raw::DataType>(data_type_id)
                    .expect("failed to read data type string");
                DataType::try_from(raw_data_type).expect("failed to parse data type")
            }),
        );

        let raw_entity_type = serde_json::from_str::<raw::EntityType>(entity_type)
            .expect("failed to read entity type string");
        let entity_type =
            EntityType::try_from(raw_entity_type).expect("failed to parse entity type");

        let entity =
            serde_json::from_str::<EntityProperties>(entity).expect("failed to read entity string");

        entity.validate(&entity_type, &provider).await
    }

    pub(crate) async fn validate_property(
        property: JsonValue,
        property_type: &'static str,
        property_types: impl IntoIterator<Item = &'static str> + Send,
        data_types: impl IntoIterator<Item = &'static str> + Send,
    ) -> Result<(), Report<PropertyValidationError>> {
        install_error_stack_hooks();

        let provider = Provider::new(
            property_types.into_iter().map(|property_type_id| {
                let raw_property_type = serde_json::from_str::<raw::PropertyType>(property_type_id)
                    .expect("failed to read property type string");
                PropertyType::try_from(raw_property_type).expect("failed to parse property type")
            }),
            data_types.into_iter().map(|data_type_id| {
                let raw_data_type = serde_json::from_str::<raw::DataType>(data_type_id)
                    .expect("failed to read data type string");
                DataType::try_from(raw_data_type).expect("failed to parse data type")
            }),
        );

        let raw_property_type = serde_json::from_str::<raw::PropertyType>(property_type)
            .expect("failed to read property type string");
        let property_type =
            PropertyType::try_from(raw_property_type).expect("failed to parse property type");

        property.validate(&property_type, &provider).await
    }

    pub(crate) async fn validate_data(
        data: JsonValue,
        data_type: &'static str,
    ) -> Result<(), Report<DataValidationError>> {
        install_error_stack_hooks();

        let raw_data_type = serde_json::from_str::<raw::DataType>(data_type)
            .expect("failed to read data type string");
        let data_type = DataType::try_from(raw_data_type).expect("failed to parse data type");

        data.validate(&data_type, &()).await
    }
}
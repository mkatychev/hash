use std::fmt::{Debug, Formatter};

use serde::Deserialize;
use type_system::{DataType, EntityType, PropertyType};
use utoipa::ToSchema;

use crate::{
    knowledge::Entity,
    store::query::{Filter, QueryRecord},
    subgraph::depths::GraphResolveDepths,
};

/// Structural queries are the main entry point to read data from the Graph.
///
/// They are used to query the graph for a set of vertices and edges that match a set of filters.
/// Alongside the filters, the query can specify the depth of the query, which determines how many
/// edges the query will follow from the root vertices. The root vertices are determined by the
/// filters. For example, if the query is for all entities of a certain type, the root vertices will
/// be the entities of that type.
///
/// # Filters
///
/// [`Filter`]s are used to specify which root vertices to include in the query. They consist of a
/// variety of different types of filters, which are described in the [`Filter`] documentation. At
/// the leaf level, filters are composed of [`RecordPath`]s and [`Parameter`]s, which identify the
/// root vertices to include in the query.
///
/// Each [`RecordPath`] is a sequence of tokens, which are used to traverse the graph. For example,
/// a `StructuralQuery<Entity>` with the path `["type", "version"]` will traverse the graph from an
/// entity to its type to the version. When associating the above path with a [`Parameter`] with the
/// value `1` in an equality filter, the query will return all entities whose type has version `1`
/// as a root vertex.
///
/// Depending on the type of the [`StructuralQuery`], different [`RecordPath`]s are valid. Please
/// see the documentation on the implementation of [`QueryRecord::Path`] for the valid paths for
/// each type.
///
/// # Depth
///
/// The depth of a query determines how many edges the query will follow from the root vertices. For
/// an in-depth explanation of the depth of a query, please see the documentation on
/// [`GraphResolveDepths`].
///
/// # Examples
///
/// Typically, a structural will be deserialized from a JSON request. The following examples assume,
/// that the type of the request body is `StructuralQuery<Entity>`.
///
/// This will return all entities with the latest version of the `foo` type:
///
/// ```json
/// {
///   "filter": {
///     "all": [
///       {
///         "equal": [
///           { "path": ["type", "baseUri"] },
///           { "parameter": "foo" }
///         ]
///       },
///       {
///         "equal": [
///           { "path": ["type", "version"] },
///           { "parameter": "latest" }
///         ]
///       }
///     ]
///   },
///   "graphResolveDepths": {
///     "dataTypeResolveDepth": 0,
///     "propertyTypeResolveDepth": 0,
///     "entityTypeResolveDepth": 0,
///     "linkTargetEntityResolveDepth": 0,
///     "linkResolveDepth": 0
///   }
/// }
/// ```
///
/// This query will return any entity, which was either created by or is owned by the account
/// `12345678-90ab-cdef-1234-567890abcdef`:
///
/// ```json
/// {
///   "filter": {
///     "any": [
///       {
///         "equal": [
///           { "path": ["createdById"] },
///           { "parameter": "12345678-90ab-cdef-1234-567890abcdef" }
///         ]
///       },
///       {
///         "equal": [
///           { "path": ["ownedById"] },
///           { "parameter": "12345678-90ab-cdef-1234-567890abcdef" }
///         ]
///       }
///     ]
///   },
///   "graphResolveDepths": {
///     "dataTypeResolveDepth": 0,
///     "propertyTypeResolveDepth": 0,
///     "entityTypeResolveDepth": 0,
///     "linkTargetEntityResolveDepth": 0,
///     "linkResolveDepth": 0
///   }
/// }
/// ```
///
/// [`RecordPath`]: crate::store::query::RecordPath
/// [`Parameter`]: crate::store::query::Parameter
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[aliases(
    DataTypeStructuralQuery = StructuralQuery<'static, DataType>,
    PropertyTypeStructuralQuery = StructuralQuery<'static, PropertyType>,
    EntityTypeStructuralQuery = StructuralQuery<'static, EntityType>,
    EntityStructuralQuery = StructuralQuery<'static, Entity>,
)]
pub struct StructuralQuery<'q, T: QueryRecord> {
    #[serde(bound = "'de: 'q, T::Path<'q>: Deserialize<'de>")]
    pub filter: Filter<'q, T>,
    pub graph_resolve_depths: GraphResolveDepths,
}

// TODO: Derive traits when bounds are generated correctly
//   see https://github.com/rust-lang/rust/issues/26925
impl<'q, T> Debug for StructuralQuery<'q, T>
where
    T: QueryRecord<Path<'q>: Debug>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructuralQuery")
            .field("filter", &self.filter)
            .field("graph_resolve_depths", &self.graph_resolve_depths)
            .finish()
    }
}

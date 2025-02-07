use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub type SubgraphQueryDepth = u8;

// TODO: update this doc: https://app.asana.com/0/1200211978612931/1203250001255262/f
/// The distance in the [`Subgraph`] to explore when searching from a root in a breadth-first search
/// manner.
///
/// Elements in the [`Subgraph`] are connected via [`Edges`]. For example, ontology elements may
/// have references to other records, a [`PropertyType`] may reference other [`PropertyType`]s or
/// [`DataType`]s. The depths provided alongside a query specify how many steps to explore along a
/// chain of references _of a certain edge kind_.
// TODO: update this to refer to specific `EdgeKind`s
/// Meaning, any chain of property type references
/// will be resolved up to the depth given for property types, and *each* data type referenced in
/// those property types will in turn start a 'new chain' whose exploration depth is limited by the
/// depth given for data types.
///
/// A depth of `0` means that no edges are explored for that edge kind.
///
/// [`Subgraph`]: crate::subgraph::Subgraph
/// [`DataType`]: type_system::DataType
/// [`PropertyType`]: type_system::PropertyType
/// [`Edges`]: crate::subgraph::edges::Edges
///
/// # Example
///
/// - `Entity1` links to `Entity2` via `Link1`
/// - `Entity2` links to `Entity3` via `Link2`
/// - `EntityType1` references \[`EntityType2`, `PropertyType1`]
/// - `EntityType2` references \[`PropertyType2`]
/// - `PropertyType1` references \[`DataType2`]
/// - `PropertyType2` references \[`PropertyType3`, `DataType1`]
/// - `PropertyType3` references \[`PropertyType4`, `DataType3`]
/// - `PropertyType4` references \[`DataType3`]
///
/// If a query on `EntityType1` is made with the following depths:
/// - `data_type_resolve_depth: 1`
/// - `property_type_resolve_depth: 3`
/// - `entity_type_resolve_depth: 1`
/// - `entity_resolve_depth: 2`
///
/// then the returned subgraph will contain the following vertices in addition to the root edges:
/// - \[`EntityType2`]
/// - \[`PropertyType1`, `PropertyType2`, `PropertyType3`]
/// - \[`DataType1`, `DataType2`]
/// - \[`Link1`, `Entity2`]
///
/// ## The idea of "chains"
///
/// When `EntityType2` is explored its referenced property types get explored. The chain of
/// _property type_ references is then resolved to a depth of `property_type_resolve_depth`. `Link2`
/// will not be included in the subgraph, because the depth for `entity_resolve_depth` is `2`
/// and `Link2` is `3` edges away from `Entity1`.

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphResolveDepths {
    #[schema(value_type = i64)]
    pub data_type_resolve_depth: SubgraphQueryDepth,
    #[schema(value_type = i64)]
    pub property_type_resolve_depth: SubgraphQueryDepth,
    #[schema(value_type = i64)]
    pub entity_type_resolve_depth: SubgraphQueryDepth,
    #[schema(value_type = i64)]
    pub entity_resolve_depth: SubgraphQueryDepth,
}

impl GraphResolveDepths {
    #[must_use]
    pub const fn zeroed() -> Self {
        Self {
            data_type_resolve_depth: 0,
            property_type_resolve_depth: 0,
            entity_type_resolve_depth: 0,
            entity_resolve_depth: 0,
        }
    }
}

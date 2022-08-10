//! The Axum webserver for accessing the Graph API operations.
//!
//! Handler methods are grouped by routes that make up the REST API.

mod api_resource;
mod data_type;
mod entity;
mod entity_type;
mod link;
mod link_type;
mod property_type;

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use include_dir::{include_dir, Dir};
use utoipa::{openapi, Modify, OpenApi};

use self::api_resource::RoutedResource;
use crate::store::{crud::Read, StorePool};

static STATIC_SCHEMAS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/api/rest/json_schemas");

fn api_resources<P: StorePool + Send + 'static>() -> Vec<Router> {
    vec![
        data_type::DataTypeResource::routes::<P>(),
        property_type::PropertyTypeResource::routes::<P>(),
        link_type::LinkTypeResource::routes::<P>(),
        entity_type::EntityTypeResource::routes::<P>(),
        entity::EntityResource::routes::<P>(),
        link::LinkResource::routes::<P>(),
    ]
}

fn api_documentation() -> Vec<openapi::OpenApi> {
    vec![
        data_type::DataTypeResource::documentation(),
        property_type::PropertyTypeResource::documentation(),
        link_type::LinkTypeResource::documentation(),
        entity_type::EntityTypeResource::documentation(),
        entity::EntityResource::documentation(),
        link::LinkResource::documentation(),
    ]
}

async fn read_from_store<'pool, P, T>(
    pool: &'pool P,
    query: &<P::Store<'pool> as Read<T>>::Query<'_>,
) -> Result<Vec<T>, StatusCode>
where
    P: StorePool,
    P::Store<'pool>: Read<T>,
    T: Send,
{
    let store = pool.acquire().await.map_err(|report| {
        tracing::error!(error=?report, "Could not acquire access to the store");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO: Implement `Valuable` for queries and print them here
    store.read(query).await.map_err(|report| {
        tracing::error!(error=?report, ?query, "Could not read from the store");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub fn rest_api_router<P: StorePool + Send + 'static>(store: Arc<P>) -> Router {
    // All api resources are merged together into a super-router.
    let merged_routes = api_resources::<P>()
        .into_iter()
        .fold(Router::new(), axum::Router::merge);

    // OpenAPI documentation is also generated by merging resources
    let open_api_doc = OpenApiDocumentation::openapi();

    // super-router can then be used as any other router.
    merged_routes
        // Make sure extensions are added at the end so they are made available to merged routers.
        .layer(Extension(store))
        .nest("/api-doc", Router::new().route(
            "/openapi.json",
            get({
                let doc = open_api_doc;
                move || async { Json(doc) }
            })).route("/models/*path", get(serve_static_schema)),
        )
}

#[allow(
    clippy::unused_async,
    reason = "This route does not need async capabilities, but axum requires it in trait bounds."
)]
async fn serve_static_schema(Path(path): Path<String>) -> Result<Response, StatusCode> {
    let path = path.trim_start_matches('/');

    match STATIC_SCHEMAS.get_file(path) {
        None => Err(StatusCode::NOT_FOUND),
        Some(file) => Ok((
            [(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            )],
            file.contents(),
        )
            .into_response()),
    }
}

#[derive(OpenApi)]
#[openapi(
        tags(
            (name = "Graph", description = "HASH Graph API")
        ),
        modifiers(&MergeAddon, &ExternalRefAddon, &OperationGraphTagAddon)
    )]
struct OpenApiDocumentation;

/// Addon to merge multiple [`OpenApi`] documents together.
///
/// [`OpenApi`]: utoipa::openapi::OpenApi
struct MergeAddon;

impl Modify for MergeAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let api_documentation = api_documentation();

        let api_components = api_documentation
            .iter()
            .cloned()
            .filter_map(|api_docs| {
                api_docs
                    .components
                    .map(|components| components.schemas.into_iter())
            })
            .flatten();

        let mut components = openapi.components.take().unwrap_or_default();
        components.schemas.extend(api_components);
        openapi.components = Some(components);

        let mut tags = openapi.tags.take().unwrap_or_default();
        tags.extend(
            api_documentation
                .iter()
                .cloned()
                .filter_map(|api_docs| api_docs.tags)
                .flatten(),
        );
        openapi.tags = Some(tags);

        openapi.paths.paths.extend(
            api_documentation
                .iter()
                .cloned()
                .flat_map(|api_docs| api_docs.paths.paths.into_iter()),
        );
    }
}

/// Addon to allow external references in schemas.
///
/// Any component that starts with `VAR_` will transform into a relative URL in the schema and
/// receive a `.json` ending.
///
/// For example the `VAR_Entity` component will be transformed into `./models/Entity.json`
struct ExternalRefAddon;

impl Modify for ExternalRefAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        for path_item in openapi.paths.paths.values_mut() {
            for operation in path_item.operations.values_mut() {
                if let Some(request_body) = &mut operation.request_body {
                    modify_component_references(&mut request_body.content);
                }

                for response in &mut operation.responses.responses.values_mut() {
                    modify_component_references(&mut response.content);
                }
            }
        }

        if let Some(components) = &mut openapi.components {
            for component in &mut components.schemas.values_mut() {
                modify_schema_references(component);
            }
        }
    }
}

fn modify_component_references(content: &mut HashMap<String, openapi::Content>) {
    for content in content.values_mut() {
        modify_schema_references(&mut content.schema);
    }
}

fn modify_schema_references(schema_component: &mut openapi::Component) {
    match schema_component {
        openapi::Component::Ref(reference) => modify_reference(reference),
        openapi::Component::Object(object) => object
            .properties
            .values_mut()
            .for_each(modify_schema_references),
        openapi::Component::Array(array) => modify_schema_references(array.items.as_mut()),
        openapi::Component::OneOf(one_of) => {
            one_of.items.iter_mut().for_each(modify_schema_references);
        }
        _ => (),
    }
}

fn modify_reference(reference: &mut openapi::Ref) {
    static REF_PREFIX: &str = "#/components/schemas/VAR_";

    if reference.ref_location.starts_with(REF_PREFIX) {
        reference
            .ref_location
            .replace_range(0..REF_PREFIX.len(), "./models/");
        reference.ref_location.make_ascii_lowercase();
        reference.ref_location.push_str(".json");
    };
}

/// Append a `Graph` tag wherever a tag appears in individual routes.
///
/// When generating API clients the tags are used for grouping routes. Having the `Graph` tag on all
/// routes makes it so that every operation appear under the same `Graph` API interface.
///
/// As generators are not all created the same way, we're putting the `Graph` tag in the beginning
/// for it to take precedence. Other tags in the system are used for logical grouping of the
/// routes, which is why we don't want to entirely replace them.
struct OperationGraphTagAddon;

impl Modify for OperationGraphTagAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        let tag = "Graph";

        for path_item in openapi.paths.paths.values_mut() {
            for operation in path_item.operations.values_mut() {
                if let Some(tags) = &mut operation.tags {
                    tags.insert(0, tag.to_owned());
                }
            }
        }
    }
}

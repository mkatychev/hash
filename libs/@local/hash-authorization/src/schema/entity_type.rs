use std::error::Error;

use graph_types::account::{AccountGroupId, AccountId};
use serde::{Deserialize, Serialize};
use type_system::url::VersionedUrl;
use uuid::Uuid;

use crate::{
    schema::{
        error::{InvalidRelationship, InvalidResource},
        PublicAccess,
    },
    zanzibar::{
        types::{Relationship, Resource},
        Affiliation, Permission, Relation,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTypeNamespace {
    #[serde(rename = "graph/entity_type")]
    EntityType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct EntityTypeId(Uuid);

impl EntityTypeId {
    #[must_use]
    pub const fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[must_use]
    pub fn from_url(url: &VersionedUrl) -> Self {
        Self(Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            url.to_string().as_bytes(),
        ))
    }

    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Resource for EntityTypeId {
    type Id = Self;
    type Kind = EntityTypeNamespace;

    fn from_parts(kind: Self::Kind, id: Self::Id) -> Result<Self, impl Error> {
        match kind {
            EntityTypeNamespace::EntityType => Ok::<_, !>(id),
        }
    }

    fn into_parts(self) -> (Self::Kind, Self::Id) {
        (EntityTypeNamespace::EntityType, self)
    }

    fn to_parts(&self) -> (Self::Kind, Self::Id) {
        Resource::into_parts(*self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityTypeResourceRelation {
    Owner,
    GeneralViewer,
}

impl Affiliation<EntityTypeId> for EntityTypeResourceRelation {}
impl Relation<EntityTypeId> for EntityTypeResourceRelation {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum EntityTypePermission {
    Update,
    View,
}

impl Affiliation<EntityTypeId> for EntityTypePermission {}
impl Permission<EntityTypeId> for EntityTypePermission {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "id")]
pub enum EntityTypeSubject {
    Public,
    Account(AccountId),
    AccountGroup(AccountGroupId),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityTypeSubjectSet {
    #[default]
    Member,
}

impl Affiliation<EntityTypeSubject> for EntityTypeSubjectSet {}
impl Relation<EntityTypeSubject> for EntityTypeSubjectSet {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTypeSubjectNamespace {
    #[serde(rename = "graph/account")]
    Account,
    #[serde(rename = "graph/account_group")]
    AccountGroup,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntityTypeSubjectId {
    Uuid(Uuid),
    Asteriks(PublicAccess),
}

impl Resource for EntityTypeSubject {
    type Id = EntityTypeSubjectId;
    type Kind = EntityTypeSubjectNamespace;

    fn from_parts(kind: Self::Kind, id: Self::Id) -> Result<Self, impl Error> {
        Ok(match (kind, id) {
            (
                EntityTypeSubjectNamespace::Account,
                EntityTypeSubjectId::Asteriks(PublicAccess::Public),
            ) => Self::Public,
            (EntityTypeSubjectNamespace::Account, EntityTypeSubjectId::Uuid(id)) => {
                Self::Account(AccountId::new(id))
            }
            (EntityTypeSubjectNamespace::AccountGroup, EntityTypeSubjectId::Uuid(id)) => {
                Self::AccountGroup(AccountGroupId::new(id))
            }
            (
                EntityTypeSubjectNamespace::AccountGroup,
                EntityTypeSubjectId::Asteriks(PublicAccess::Public),
            ) => {
                return Err(InvalidResource::<Self>::invalid_id(kind, id));
            }
        })
    }

    fn into_parts(self) -> (Self::Kind, Self::Id) {
        match self {
            Self::Public => (
                EntityTypeSubjectNamespace::Account,
                EntityTypeSubjectId::Asteriks(PublicAccess::Public),
            ),
            Self::Account(id) => (
                EntityTypeSubjectNamespace::Account,
                EntityTypeSubjectId::Uuid(id.into_uuid()),
            ),
            Self::AccountGroup(id) => (
                EntityTypeSubjectNamespace::AccountGroup,
                EntityTypeSubjectId::Uuid(id.into_uuid()),
            ),
        }
    }

    fn to_parts(&self) -> (Self::Kind, Self::Id) {
        Resource::into_parts(*self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase", tag = "kind", deny_unknown_fields)]
pub enum EntityTypeOwnerSubject {
    Account {
        #[serde(rename = "subjectId")]
        id: AccountId,
    },
    AccountGroup {
        #[serde(rename = "subjectId")]
        id: AccountGroupId,
        #[serde(skip)]
        set: EntityTypeSubjectSet,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase", tag = "kind", deny_unknown_fields)]
pub enum EntityTypeGeneralViewerSubject {
    Public,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase", tag = "relation", content = "subject")]
pub enum EntityTypeRelationAndSubject {
    Owner(EntityTypeOwnerSubject),
    GeneralViewer(EntityTypeGeneralViewerSubject),
}

impl Relationship for (EntityTypeId, EntityTypeRelationAndSubject) {
    type Relation = EntityTypeResourceRelation;
    type Resource = EntityTypeId;
    type Subject = EntityTypeSubject;
    type SubjectSet = EntityTypeSubjectSet;

    fn from_parts(
        resource: Self::Resource,
        relation: Self::Relation,
        subject: Self::Subject,
        subject_set: Option<Self::SubjectSet>,
    ) -> Result<Self, impl Error> {
        Ok((
            resource,
            match relation {
                EntityTypeResourceRelation::Owner => match (subject, subject_set) {
                    (EntityTypeSubject::Account(id), None) => {
                        EntityTypeRelationAndSubject::Owner(EntityTypeOwnerSubject::Account { id })
                    }
                    (EntityTypeSubject::AccountGroup(id), Some(set)) => {
                        EntityTypeRelationAndSubject::Owner(EntityTypeOwnerSubject::AccountGroup {
                            id,
                            set,
                        })
                    }
                    (EntityTypeSubject::Public, subject_set) => {
                        return Err(InvalidRelationship::<Self>::invalid_subject(
                            resource,
                            relation,
                            subject,
                            subject_set,
                        ));
                    }
                    (
                        EntityTypeSubject::Account(_) | EntityTypeSubject::AccountGroup(_),
                        subject_set,
                    ) => {
                        return Err(InvalidRelationship::<Self>::invalid_subject_set(
                            resource,
                            relation,
                            subject,
                            subject_set,
                        ));
                    }
                },
                EntityTypeResourceRelation::GeneralViewer => match (subject, subject_set) {
                    (EntityTypeSubject::Public, None) => {
                        EntityTypeRelationAndSubject::GeneralViewer(
                            EntityTypeGeneralViewerSubject::Public,
                        )
                    }
                    (
                        EntityTypeSubject::Account(_)
                        | EntityTypeSubject::AccountGroup(_)
                        | EntityTypeSubject::Public,
                        subject_set,
                    ) => {
                        return Err(InvalidRelationship::<Self>::invalid_subject_set(
                            resource,
                            relation,
                            subject,
                            subject_set,
                        ));
                    }
                },
            },
        ))
    }

    fn to_parts(
        &self,
    ) -> (
        Self::Resource,
        Self::Relation,
        Self::Subject,
        Option<Self::SubjectSet>,
    ) {
        Self::into_parts(*self)
    }

    fn into_parts(
        self,
    ) -> (
        Self::Resource,
        Self::Relation,
        Self::Subject,
        Option<Self::SubjectSet>,
    ) {
        let (relation, (subject, subject_set)) = match self.1 {
            EntityTypeRelationAndSubject::Owner(subject) => (
                EntityTypeResourceRelation::Owner,
                match subject {
                    EntityTypeOwnerSubject::Account { id } => {
                        (EntityTypeSubject::Account(id), None)
                    }
                    EntityTypeOwnerSubject::AccountGroup { id, set } => {
                        (EntityTypeSubject::AccountGroup(id), Some(set))
                    }
                },
            ),
            EntityTypeRelationAndSubject::GeneralViewer(subject) => (
                EntityTypeResourceRelation::GeneralViewer,
                match subject {
                    EntityTypeGeneralViewerSubject::Public => (EntityTypeSubject::Public, None),
                },
            ),
        };
        (self.0, relation, subject, subject_set)
    }
}
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    pub summary: Option<String>,
    pub comment: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::blob::Entity")]
    Blob,
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::item_tag::Relation::Tag.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::item_tag::Relation::Item.def().rev())
    }
}

impl Related<super::blob::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Blob.def()
    }
}

#[derive(Debug)]
pub struct LinkedItem;

impl Linked for LinkedItem {
    type FromEntity = Entity;
    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::link::Relation::A.def().rev(),
            super::link::Relation::B.def(),
        ]
    }
}

// impl Related<Entity> for Entity {
//     fn to() -> RelationDef {
//         super::link::Relation::B.def()
//     }

//     fn via() -> Option<RelationDef> {
//         Some(super::link::Relation::B.def().rev())
//     }
// }

impl ActiveModelBehavior for ActiveModel {}

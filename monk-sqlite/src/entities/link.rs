use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "link")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub a_id: Uuid,
    #[sea_orm(primary_key)]
    pub b_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    A,
    B,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::A => Entity::belongs_to(super::item::Entity)
                .from(Column::AId)
                .to(super::item::Column::Id)
                .into(),
            Self::B => Entity::belongs_to(super::item::Entity)
                .from(Column::BId)
                .to(super::item::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}

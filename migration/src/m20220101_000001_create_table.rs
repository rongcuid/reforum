use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DbBackend, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::Password).string())
                    .col(ColumnDef::new(Users::Signature).string())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .date_time()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(ColumnDef::new(Users::UpdatedAt).date_time())
                    .col(ColumnDef::new(Users::LastSeenAt).date_time())
                    .col(ColumnDef::new(Users::LastPostAt).date_time())
                    .col(ColumnDef::new(Users::MutedUntil).date_time())
                    .col(ColumnDef::new(Users::BannedAt).date_time())
                    .to_owned(),
            )
            .await?;
        // Moderators
        manager
            .create_table(
                Table::create()
                    .table(Moderators::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Moderators::UserId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Moderators::AssignedAt)
                            .date_time()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Moderators::Table, Moderators::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        // Past Moderators
        manager
            .create_table(
                Table::create()
                    .table(PastModerators::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PastModerators::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PastModerators::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(PastModerators::UnassignedAt)
                            .date_time()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(ColumnDef::new(PastModerators::Reason).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PastModerators::Table, PastModerators::UserId)
                            .to(Users::Table, Users::Id)
                            // These logs are NEVER deleted!
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        // Topics
        manager
            .create_table(
                Table::create()
                    .table(Topics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Topics::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Topics::AuthorUserId).integer().not_null())
                    .col(ColumnDef::new(Topics::Title).text().not_null())
                    .col(
                        ColumnDef::new(Topics::NumberPosts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Topics::Public)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Topics::CreatedAt)
                            .date_time()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(ColumnDef::new(Topics::UpdatedAt).date_time())
                    .col(ColumnDef::new(Topics::DeletedAt).date_time())
                    .col(
                        ColumnDef::new(Topics::ViewsFromUsers)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Topics::Table, Topics::AuthorUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        // Posts
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Posts::TopicId).integer().not_null())
                    .col(ColumnDef::new(Posts::AuthorUserId).integer().not_null())
                    .col(ColumnDef::new(Posts::Body).text().not_null())
                    .col(ColumnDef::new(Posts::PostNumber).integer().not_null())
                    .col(
                        ColumnDef::new(Posts::Public)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .date_time()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(ColumnDef::new(Posts::UpdatedAt).date_time())
                    .col(ColumnDef::new(Posts::DeletedAt).date_time())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::AuthorUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::TopicId)
                            .to(Topics::Table, Topics::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        // Replies
        manager
            .create_table(
                Table::create()
                    .table(Replies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Replies::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Replies::PostId).integer().not_null())
                    .col(ColumnDef::new(Replies::AuthorUserId).integer().not_null())
                    .col(ColumnDef::new(Replies::Body).text().not_null())
                    .col(
                        ColumnDef::new(Replies::CreatedAt)
                            .date_time()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Replies::Table, Replies::AuthorUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Replies::Table, Replies::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        // Sessions
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sessions::Id)
                            .blob(BlobSize::Tiny)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sessions::UserId).integer().not_null())
                    .col(ColumnDef::new(Sessions::ExpiresAt).date_time())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Sessions::Table, Sessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        match manager.get_database_backend() {
            DbBackend::Sqlite => {
                manager
                    .get_connection()
                    .execute(Statement::from_string(
                        DbBackend::Sqlite,
                        include_str!("sqlite_init.sql").to_owned(),
                    ))
                    .await?;
            }
            _ => {
                Err(DbErr::Migration("Unsupported DB".to_owned()))?;
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PastModerators::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Moderators::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Replies::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Topics::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Users {
    Table,
    Id,
    Name,
    Password,
    Signature,
    CreatedAt,
    UpdatedAt,
    LastSeenAt,
    LastPostAt,
    MutedUntil,
    BannedAt,
}

#[derive(Iden)]
enum Moderators {
    Table,
    UserId,
    AssignedAt,
}

#[derive(Iden)]
enum PastModerators {
    Table,
    Id,
    UserId,
    UnassignedAt,
    Reason,
}

#[derive(Iden)]
enum Topics {
    Table,
    Id,
    AuthorUserId,
    Title,
    NumberPosts,
    Public,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    ViewsFromUsers,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
    TopicId,
    AuthorUserId,
    Body,
    PostNumber,
    Public,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Replies {
    Table,
    Id,
    PostId,
    AuthorUserId,
    Body,
    CreatedAt,
}

#[derive(Iden)]
enum Sessions {
    Table,
    Id,
    UserId,
    ExpiresAt,
}

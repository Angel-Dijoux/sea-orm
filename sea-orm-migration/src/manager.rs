use super::{IntoSchemaManagerConnection, SchemaManagerConnection};
use sea_orm::sea_query::{
    extension::postgres::{TypeAlterStatement, TypeCreateStatement, TypeDropStatement},
    ForeignKeyCreateStatement, ForeignKeyDropStatement, IndexCreateStatement, IndexDropStatement,
    TableAlterStatement, TableCreateStatement, TableDropStatement, TableRenameStatement,
    TableTruncateStatement,
};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, StatementBuilder};
use sea_schema::{mysql::MySql, postgres::Postgres, probe::SchemaProbe, sqlite::Sqlite};

/// Helper struct for writing migration scripts in migration file
pub struct SchemaManager<'c> {
    conn: SchemaManagerConnection<'c>,
}

impl<'c> SchemaManager<'c> {
    pub fn new<T>(conn: T) -> Self
    where
        T: IntoSchemaManagerConnection<'c>,
    {
        Self {
            conn: conn.into_schema_manager_connection(),
        }
    }

    pub async fn exec_stmt<S>(&self, stmt: S) -> Result<(), DbErr>
    where
        S: StatementBuilder,
    {
        let builder = self.conn.get_database_backend();
        self.conn.execute(builder.build(&stmt)).await.map(|_| ())
    }

    pub fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }

    pub fn get_connection(&self) -> &SchemaManagerConnection<'c> {
        &self.conn
    }
}

/// Schema Creation
impl SchemaManager<'_> {
    pub async fn create_table(&self, stmt: TableCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn create_index(&self, stmt: IndexCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn create_foreign_key(&self, stmt: ForeignKeyCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn create_type(&self, stmt: TypeCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }
}

/// Schema Mutation
impl SchemaManager<'_> {
    pub async fn alter_table(&self, stmt: TableAlterStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_table(&self, stmt: TableDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn rename_table(&self, stmt: TableRenameStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn truncate_table(&self, stmt: TableTruncateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_index(&self, stmt: IndexDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_foreign_key(&self, stmt: ForeignKeyDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn alter_type(&self, stmt: TypeAlterStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_type(&self, stmt: TypeDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }
}

/// Schema Inspection.
impl SchemaManager<'_> {
    pub async fn has_table<T>(&self, table: T) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
    {
        has_table(&self.conn, table).await
    }

    pub async fn has_column<T, C>(&self, table: T, column: C) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
        C: AsRef<str>,
    {
        let stmt = match self.conn.get_database_backend() {
            DbBackend::MySql => MySql.has_column(table, column),
            DbBackend::Postgres => Postgres.has_column(table, column),
            DbBackend::Sqlite => Sqlite.has_column(table, column),
        };

        let builder = self.conn.get_database_backend();
        let res = self
            .conn
            .query_one(builder.build(&stmt))
            .await?
            .ok_or_else(|| DbErr::Custom("Failed to check column exists".to_owned()))?;

        res.try_get("", "has_column")
    }

    pub async fn has_index<T, I>(&self, table: T, index: I) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
        I: AsRef<str>,
    {
        let stmt = match self.conn.get_database_backend() {
            DbBackend::MySql => MySql.has_index(table, index),
            DbBackend::Postgres => Postgres.has_index(table, index),
            DbBackend::Sqlite => Sqlite.has_index(table, index),
        };

        let builder = self.conn.get_database_backend();
        let res = self
            .conn
            .query_one(builder.build(&stmt))
            .await?
            .ok_or_else(|| DbErr::Custom("Failed to check index exists".to_owned()))?;

        res.try_get("", "has_index")
    }
}

pub(crate) async fn has_table<C, T>(conn: &C, table: T) -> Result<bool, DbErr>
where
    C: ConnectionTrait,
    T: AsRef<str>,
{
    let stmt = match conn.get_database_backend() {
        DbBackend::MySql => MySql.has_table(table),
        DbBackend::Postgres => Postgres.has_table(table),
        DbBackend::Sqlite => Sqlite.has_table(table),
    };

    let builder = conn.get_database_backend();
    let res = conn
        .query_one(builder.build(&stmt))
        .await?
        .ok_or_else(|| DbErr::Custom("Failed to check table exists".to_owned()))?;

    res.try_get("", "has_table")
}

use std::sync::Arc;

use async_sqlite::{rusqlite::types::FromSql, Client};

use crate::{
    sql::{SelectBuilder, SqlValues, UsedColumns},
    sql_insert::SqlInsertModel,
    sql_select::SelectEntity,
    sql_update::SqlUpdateModel,
    sql_where::SqlWhereModel,
    CountResult, DbRow, SqlLiteError,
};

pub struct SqlLiteConnection {
    pub client: Client,
}

impl SqlLiteConnection {
    pub async fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn insert_db_entity<TEntity: SqlInsertModel>(
        &self,
        entity: &TEntity,
        table_name: &str,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<usize, SqlLiteError> {
        let sql_data = crate::sql::build_insert_sql(
            crate::sql::InsertType::JustInsert,
            entity,
            table_name,
            &mut UsedColumns::as_none(),
        );

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql_data.sql);
        }

        let sql_data = Arc::new(sql_data);

        let sql_data_spawned = sql_data.clone();

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data_spawned.sql,
                    sql_data_spawned.values.get_params_to_invoke().as_slice(),
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql_data.sql);
        }

        let result = result?;

        Ok(result)
    }

    pub async fn insert_db_entity_if_not_exists<TEntity: SqlInsertModel>(
        &self,
        entity: &TEntity,
        table_name: &str,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<usize, SqlLiteError> {
        let sql_data = crate::sql::build_insert_sql(
            crate::sql::InsertType::OrIgnore,
            entity,
            table_name,
            &mut UsedColumns::as_none(),
        );

        let sql_data = Arc::new(sql_data);

        let sql_data_spawned = sql_data.clone();

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql_data.sql);
        }

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data_spawned.sql,
                    sql_data_spawned.values.get_params_to_invoke().as_slice(),
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql_data.sql);
        }

        Ok(result?)
    }

    pub async fn insert_or_update_db_entity<'s, TEntity: SqlInsertModel + SqlUpdateModel>(
        &self,
        table_name: &str,
        entity: &TEntity,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<usize, SqlLiteError> {
        let sql_data = crate::sql::build_insert_or_update_sql(entity, table_name);

        let sql_data = Arc::new(sql_data);

        let sql_data_spawned = sql_data.clone();

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql_data.sql);
        }

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data_spawned.sql,
                    sql_data_spawned.values.get_params_to_invoke().as_slice(),
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql_data.sql);
        }

        Ok(result?)
    }

    pub async fn bulk_insert_db_entities<TEntity: SqlInsertModel>(
        &self,
        entities: &[TEntity],
        table_name: &str,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), SqlLiteError> {
        if entities.len() == 0 {
            panic!("Attempt to bulk_insert_db_entities 0 entities");
        }

        let used_columns = entities[0].get_insert_columns_list();
        let sql_data = Arc::new(crate::sql::build_bulk_insert_sql(
            crate::sql::InsertType::JustInsert,
            entities,
            table_name,
            &used_columns,
        ));

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql_data.sql);
        }

        let sql_data_spawned = sql_data.clone();

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data_spawned.sql,
                    sql_data_spawned.values.get_params_to_invoke().as_slice(),
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql_data.sql);
        }

        Ok(())
    }

    pub async fn bulk_insert_or_update<'s, TEntity: SqlInsertModel + SqlUpdateModel>(
        &self,
        entities: &[TEntity],
        table_name: &str,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), SqlLiteError> {
        let sql_data = crate::sql::build_bulk_insert_or_update_sql(table_name, entities);

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql_data.sql);
        }

        let sql_data = Arc::new(sql_data);

        let sql_data_spawned = sql_data.clone();

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data_spawned.sql,
                    sql_data_spawned.values.get_params_to_invoke().as_slice(),
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql_data.sql);
        }

        Ok(())
    }

    pub async fn bulk_insert_db_entities_if_not_exists<TEntity: SqlInsertModel>(
        &self,
        entities: &[TEntity],
        table_name: &str,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), SqlLiteError> {
        if entities.len() == 0 {
            panic!("Attempt to bulk_insert_db_entities 0 entities");
        }

        let used_columns = entities[0].get_insert_columns_list();
        let sql_data = Arc::new(crate::sql::build_bulk_insert_sql(
            crate::sql::InsertType::OrIgnore,
            entities,
            table_name,
            &used_columns,
        ));

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql_data.sql);
        }

        let sql_data_spawned = sql_data.clone();

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data_spawned.sql,
                    sql_data_spawned.values.get_params_to_invoke().as_slice(),
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql_data.sql);
        }

        Ok(())
    }

    pub async fn query_rows<
        TEntity: SelectEntity + Send + Sync + 'static,
        TWhereModel: SqlWhereModel,
    >(
        &self,
        table_name: &str,
        where_model: Option<&TWhereModel>,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<Vec<TEntity>, SqlLiteError> {
        let select_builder = SelectBuilder::from_select_model::<TEntity>();

        let select_fields = TEntity::get_select_fields();

        let mut sql = String::new();

        let mut sql_values = SqlValues::new();

        select_builder.build_select_sql(&mut sql, &mut sql_values, table_name, where_model);

        let sql = Arc::new(sql);

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql);
        }

        let sql_spawned = sql.clone();

        let result = self
            .client
            .conn(move |conn| {
                let mut stmt = conn.prepare(&sql_spawned)?;

                let response =
                    stmt.query_map(sql_values.get_params_to_invoke().as_slice(), |row| {
                        let db_row = DbRow::new(row, &select_fields);
                        TEntity::from(&db_row);
                        Ok(TEntity::from(&db_row))
                    })?;

                let mut result = Vec::new();

                for itm in response {
                    let itm = itm?;
                    result.push(itm);
                }

                Ok(result)
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql.as_str());
        }

        Ok(result?)
    }

    pub async fn query_single_row<
        TEntity: SelectEntity + Send + Sync + 'static,
        TWhereModel: SqlWhereModel,
    >(
        &self,
        table_name: &str,
        where_model: Option<&TWhereModel>,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<Option<TEntity>, SqlLiteError> {
        let select_builder = SelectBuilder::from_select_model::<TEntity>();

        let select_fields = TEntity::get_select_fields();

        let mut sql = String::new();

        #[cfg(test)]
        println!("Sql: {}", sql);

        let mut sql_values = SqlValues::new();

        select_builder.build_select_sql(&mut sql, &mut sql_values, table_name, where_model);

        let sql = Arc::new(sql);

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql);
        }

        let sql_spawned = sql.clone();

        let result = self
            .client
            .conn(move |conn| {
                conn.query_row_and_then(
                    &sql_spawned,
                    sql_values.get_params_to_invoke().as_slice(),
                    |row| {
                        let db_row = DbRow::new(row, &select_fields);
                        TEntity::from(&db_row);
                        Ok(TEntity::from(&db_row))
                    },
                )
            })
            .await;

        if let Err(err) = &result {
            match err {
                async_sqlite::Error::Rusqlite(err) => match err {
                    async_sqlite::rusqlite::Error::QueryReturnedNoRows => {
                        return Ok(None);
                    }

                    _ => {}
                },
                _ => {}
            }

            println!("Sql: {}", sql.as_str());
        }

        let value = result?;

        Ok(Some(value))
    }

    pub async fn get_count<
        TWhereModel: SqlWhereModel,
        TResult: CountResult + FromSql + Send + Sync + 'static,
    >(
        &self,
        table_name: &str,
        where_model: Option<&TWhereModel>,
        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<Option<TResult>, SqlLiteError> {
        let mut sql = String::new();

        let mut sql_values = SqlValues::new();
        sql.push_str("SELECT COUNT(*)::");
        sql.push_str(TResult::get_postgres_type());

        sql.push_str(" FROM ");
        sql.push_str(table_name);

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql);
        }

        if let Some(where_model) = where_model {
            if where_model.has_conditions() {
                sql.push_str(" WHERE ");
                where_model.fill_where_component(&mut sql, &mut sql_values);
            }

            where_model.fill_limit_and_offset(&mut sql);
        }

        let sql = Arc::new(sql);

        let sql_spawned = sql.clone();

        let result = self
            .client
            .conn(move |conn| {
                conn.query_row_and_then(
                    &sql_spawned,
                    sql_values.get_params_to_invoke().as_slice(),
                    |row| {
                        let result = row.get(0)?;
                        Ok(result)
                    },
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql.as_str());
        }

        Ok(result?)
    }

    pub async fn delete_db_entity<TWhereModel: SqlWhereModel>(
        &self,
        table_name: &str,
        where_model: &TWhereModel,

        #[cfg(feature = "with-logs-and-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), SqlLiteError> {
        let sql_data = where_model.build_delete_sql(table_name);

        if std::env::var("DEBUG").is_ok() {
            println!("Sql: {}", sql_data.sql);
        }

        let sql_data = Arc::new(sql_data);

        let sql_data_spawned = sql_data.clone();

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data_spawned.sql,
                    sql_data_spawned.values.get_params_to_invoke().as_slice(),
                )
            })
            .await;

        if result.is_err() {
            println!("Sql: {}", sql_data.sql);
        }

        Ok(())
    }
}

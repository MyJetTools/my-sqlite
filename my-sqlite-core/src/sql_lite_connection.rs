use std::sync::Arc;

use async_sqlite::{rusqlite::types::FromSql, Client};

use crate::{
    sql::{SelectBuilder, SqlValues, UsedColumns},
    sql_insert::SqlInsertModel,
    sql_select::SelectEntity,
    sql_update::SqlUpdateModel,
    sql_where::SqlWhereModel,
    table_schema::TableSchemaProvider,
    CountResult, DbRow, SqlLiteError,
};

pub struct SqlLiteConnection {
    pub client: Client,
    debug: bool,
}

impl SqlLiteConnection {
    pub async fn new(client: Client, debug: bool) -> Self {
        Self { client, debug }
    }

    fn is_debug(&self) -> bool {
        self.debug
    }

    pub async fn create_table_if_not_exists<T: TableSchemaProvider>(
        &self,
        table_name: &str,
    ) -> Result<(), SqlLiteError> {
        let crate_table_sql = crate::crate_table::generate_sql_request::<T>(table_name);

        self.client
            .conn(move |connection| connection.execute(crate_table_sql.as_str(), []))
            .await?;

        if let Some(indexes) = T::get_indexes() {
            for (name, index_schema) in indexes {
                let index_sql = crate::crate_table::generate_create_index_sql(
                    table_name,
                    name.as_str(),
                    index_schema,
                );

                self.client
                    .conn(move |connection| connection.execute(index_sql.as_str(), []))
                    .await?;
            }
        }

        Ok(())
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql_data.sql);
            }
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql_data.sql);
            }
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql_data.sql);
            }
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql_data.sql);
            }
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql_data.sql);
            }
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql_data.sql);
            }
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql);
            }
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

        if self.is_debug() {
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql);
            }
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

        if self.is_debug() {
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

        if let Err(err) = &result {
            println!("Err: {}", err);
            if self.is_debug() {
                println!("Sql: {}", sql_data.sql);
            }
        }

        Ok(())
    }

    // Connection can not be reused after
    pub async fn close(&self) {
        let close_result = self.client.close().await;

        if let Err(close_result) = close_result {
            println!("Error closing sqlite connection: {}", close_result);
        }
    }
}

use async_sqlite::{rusqlite::types::FromSql, Client};

use crate::{
    sql::{SelectBuilder, SqlValues, UsedColumns},
    sql_insert::SqlInsertModel,
    sql_select::SelectEntity,
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
        let sql_data =
            crate::sql::build_insert_sql(entity, table_name, &mut UsedColumns::as_none());

        #[cfg(test)]
        println!("Sql: {}", sql_data.sql);

        let result = self
            .client
            .conn(move |conn| {
                conn.execute(
                    &sql_data.sql,
                    sql_data.values.get_params_to_invoke().as_slice(),
                )
            })
            .await?;

        Ok(result)
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
        let sql_data = crate::sql::build_bulk_insert_sql(entities, table_name, &used_columns);

        #[cfg(test)]
        println!("Sql: {}", sql_data.sql);

        self.client
            .conn(move |conn| {
                conn.execute(
                    &sql_data.sql,
                    sql_data.values.get_params_to_invoke().as_slice(),
                )
            })
            .await?;

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

        let value = self
            .client
            .conn(move |conn| {
                let mut stmt = conn.prepare(&sql)?;

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
            .await?;

        Ok(value)
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

        let result = self
            .client
            .conn(move |conn| {
                conn.query_row_and_then(&sql, sql_values.get_params_to_invoke().as_slice(), |row| {
                    let db_row = DbRow::new(row, &select_fields);
                    TEntity::from(&db_row);
                    Ok(TEntity::from(&db_row))
                })
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

        if let Some(where_model) = where_model {
            if where_model.has_conditions() {
                sql.push_str(" WHERE ");
                where_model.fill_where_component(&mut sql, &mut sql_values);
            }

            where_model.fill_limit_and_offset(&mut sql);
        }

        let result = self
            .client
            .conn(move |conn| {
                conn.query_row_and_then(&sql, sql_values.get_params_to_invoke().as_slice(), |row| {
                    let result = row.get(0)?;
                    Ok(result)
                })
            })
            .await?;

        Ok(result)
    }
}

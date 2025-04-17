use std::sync::Arc;

use async_sqlite::Client;
use rust_extensions::StrOrString;

use crate::{
    sql::{SelectBuilder, SqlValues},
    sql_select::SelectEntity,
    sql_where::SqlWhereModel,
    DbRow, SqlLiteError,
};

pub struct SqliteQueryStream<TEntity: SelectEntity + Send + Sync + 'static> {
    rx: tokio::sync::mpsc::Receiver<Result<TEntity, async_sqlite::rusqlite::Error>>,
}

impl<TEntity: SelectEntity + Send + Sync + 'static> SqliteQueryStream<TEntity> {
    pub fn new<TWhereModel: SqlWhereModel + Send + Sync + 'static>(
        client: Arc<Client>,
        table_name: StrOrString<'static>,
        select_builder: SelectBuilder,
        where_model: Option<TWhereModel>,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(2048);
        tokio::spawn(select_builder_stream::<TEntity, TWhereModel>(
            client,
            table_name,
            select_builder,
            where_model,
            tx,
        ));
        Self { rx }
    }

    pub async fn get_next(&mut self) -> Option<Result<TEntity, SqlLiteError>> {
        let next_one = self.rx.recv().await?;

        match next_one {
            Ok(item) => Some(Ok(item)),
            Err(err) => Some(Err(err.into())),
        }
    }
}

async fn select_builder_stream<
    TEntity: SelectEntity + Send + Sync + 'static,
    TWhereModel: SqlWhereModel,
>(
    client: Arc<Client>,
    table_name: StrOrString<'static>,
    select_builder: SelectBuilder,
    where_model: Option<TWhereModel>,
    tx: tokio::sync::mpsc::Sender<Result<TEntity, async_sqlite::rusqlite::Error>>,
) {
    let mut sql = String::new();

    let mut sql_values = SqlValues::new();

    select_builder.build_select_sql(
        &mut sql,
        &mut sql_values,
        table_name.as_str(),
        where_model.as_ref(),
    );

    let sql = Arc::new(sql);

    let sql_spawned = sql.clone();

    let result = client
        .conn(move |conn| {
            let mut stmt = conn.prepare(&sql_spawned)?;

            let response = stmt.query_map(sql_values.get_params_to_invoke().as_slice(), |row| {
                let db_row = DbRow::new(row, TEntity::SELECT_FIELDS);
                TEntity::from(&db_row);
                Ok(TEntity::from(&db_row))
            })?;

            for itm in response {
                let send_result = tx.blocking_send(itm);

                if let Err(err) = send_result {
                    println!(
                        "Sending DbRow to string ended with Error. Err:{:?}. Sql:{}",
                        err, sql_spawned
                    );
                }
            }

            Ok(())
        })
        .await;

    if let Err(err) = result {
        println!("Reading stream ended with error. Err:{:?} Sql:{}", err, sql);
    }
}

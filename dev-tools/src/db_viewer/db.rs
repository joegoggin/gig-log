use std::collections::BTreeMap;

use anyhow::Context;
use serde_json::{Map, Value};
use sqlx::{Column, PgPool, Row};

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub udt_name: String,
    pub is_nullable: bool,
    pub enum_options: Vec<String>,
}

impl ColumnInfo {
    pub fn is_enum(&self) -> bool {
        !self.enum_options.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub ctid: String,
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum QueryOutput {
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        row_count: usize,
    },
    Command {
        rows_affected: u64,
    },
}

fn quote_ident(raw: &str) -> String {
    let escaped = raw.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn qualified_table(schema: &str, table: &str) -> String {
    format!("{}.{}", quote_ident(schema), quote_ident(table))
}

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    PgPool::connect(database_url)
        .await
        .context("failed to connect to Postgres")
}

pub async fn load_tables(pool: &PgPool) -> anyhow::Result<Vec<TableInfo>> {
    let table_rows = sqlx::query(
        r#"
        SELECT table_schema, table_name
        FROM information_schema.tables
        WHERE table_type = 'BASE TABLE'
          AND table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY table_schema, table_name
        "#,
    )
    .fetch_all(pool)
    .await
    .context("failed to load table list")?;

    let mut tables = Vec::with_capacity(table_rows.len());

    for row in table_rows {
        let schema: String = row.try_get("table_schema")?;
        let name: String = row.try_get("table_name")?;

        let column_rows = sqlx::query(
            r#"
            SELECT
                column_name,
                data_type,
                udt_name,
                udt_schema,
                is_nullable = 'YES' AS is_nullable
            FROM information_schema.columns
            WHERE table_schema = $1
              AND table_name = $2
            ORDER BY ordinal_position
            "#,
        )
        .bind(&schema)
        .bind(&name)
        .fetch_all(pool)
        .await
        .with_context(|| format!("failed to load columns for {schema}.{name}"))?;

        let mut columns = Vec::with_capacity(column_rows.len());

        for column_row in column_rows {
            let column_name: String = column_row.try_get("column_name")?;
            let data_type: String = column_row.try_get("data_type")?;
            let udt_name: String = column_row.try_get("udt_name")?;
            let udt_schema: String = column_row.try_get("udt_schema")?;
            let is_nullable: bool = column_row.try_get("is_nullable")?;

            let enum_options = sqlx::query(
                r#"
                SELECT e.enumlabel
                FROM pg_type t
                JOIN pg_enum e ON t.oid = e.enumtypid
                JOIN pg_namespace n ON n.oid = t.typnamespace
                WHERE n.nspname = $1
                  AND t.typname = $2
                ORDER BY e.enumsortorder
                "#,
            )
            .bind(&udt_schema)
            .bind(&udt_name)
            .fetch_all(pool)
            .await
            .with_context(|| {
                format!(
                    "failed to load enum labels for {schema}.{name}.{column_name} ({udt_schema}.{udt_name})"
                )
            })?
            .into_iter()
            .filter_map(|label_row| label_row.try_get::<String, _>("enumlabel").ok())
            .collect::<Vec<_>>();

            columns.push(ColumnInfo {
                name: column_name,
                data_type,
                udt_name,
                is_nullable,
                enum_options,
            });
        }

        tables.push(TableInfo {
            schema,
            name,
            columns,
        });
    }

    Ok(tables)
}

pub async fn fetch_rows(
    pool: &PgPool,
    table: &TableInfo,
    limit: i64,
) -> anyhow::Result<Vec<TableRow>> {
    let table_ref = qualified_table(&table.schema, &table.name);
    let sql = format!(
        "SELECT ctid::text AS ctid, to_jsonb(t) AS row_json FROM {table_ref} AS t LIMIT $1"
    );

    let rows = sqlx::query(&sql)
        .bind(limit)
        .fetch_all(pool)
        .await
        .with_context(|| format!("failed to load rows for {}.{}", table.schema, table.name))?;

    let mut out = Vec::with_capacity(rows.len());

    for row in rows {
        let ctid: String = row.try_get("ctid")?;
        let row_json: sqlx::types::Json<Value> = row.try_get("row_json")?;
        let values = match row_json.0 {
            Value::Object(map) => json_map_to_btree(map),
            _ => BTreeMap::new(),
        };

        out.push(TableRow { ctid, values });
    }

    Ok(out)
}

fn json_map_to_btree(map: Map<String, Value>) -> BTreeMap<String, Value> {
    map.into_iter().collect()
}

pub async fn update_row_values(
    pool: &PgPool,
    table: &TableInfo,
    ctid: &str,
    values: &BTreeMap<String, Value>,
) -> anyhow::Result<()> {
    if values.is_empty() {
        return Ok(());
    }

    let table_ref = qualified_table(&table.schema, &table.name);
    let set_clause = values
        .keys()
        .map(|column| format!("{} = r.{}", quote_ident(column), quote_ident(column)))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "UPDATE {table_ref} AS t \
         SET {set_clause} \
         FROM jsonb_populate_record(NULL::{table_ref}, $1::jsonb) AS r \
         WHERE t.ctid = $2::tid"
    );

    let payload = Value::Object(values.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

    sqlx::query(&sql)
        .bind(sqlx::types::Json(payload))
        .bind(ctid)
        .execute(pool)
        .await
        .with_context(|| {
            format!(
                "failed to update row {ctid} in {}.{}",
                table.schema, table.name
            )
        })?;

    Ok(())
}

pub async fn insert_row(
    pool: &PgPool,
    table: &TableInfo,
    values: &BTreeMap<String, Value>,
) -> anyhow::Result<()> {
    let table_ref = qualified_table(&table.schema, &table.name);

    if values.is_empty() {
        let sql = format!("INSERT INTO {table_ref} DEFAULT VALUES");
        sqlx::query(&sql)
            .execute(pool)
            .await
            .with_context(|| format!("failed to insert row in {}.{}", table.schema, table.name))?;
        return Ok(());
    }

    let columns = values
        .keys()
        .map(|column| quote_ident(column))
        .collect::<Vec<_>>()
        .join(", ");
    let selected_columns = values
        .keys()
        .map(|column| format!("r.{}", quote_ident(column)))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "INSERT INTO {table_ref} ({columns}) \
         SELECT {selected_columns} \
         FROM jsonb_populate_record(NULL::{table_ref}, $1::jsonb) AS r"
    );

    let payload = Value::Object(values.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

    sqlx::query(&sql)
        .bind(sqlx::types::Json(payload))
        .execute(pool)
        .await
        .with_context(|| format!("failed to insert row in {}.{}", table.schema, table.name))?;

    Ok(())
}

pub async fn delete_rows(
    pool: &PgPool,
    table: &TableInfo,
    ctids: &[String],
) -> anyhow::Result<u64> {
    if ctids.is_empty() {
        return Ok(0);
    }

    let table_ref = qualified_table(&table.schema, &table.name);
    let sql = format!("DELETE FROM {table_ref} WHERE ctid = $1::tid");

    let mut tx = pool
        .begin()
        .await
        .context("failed to begin delete transaction")?;
    let mut deleted = 0_u64;

    for ctid in ctids {
        let result = sqlx::query(&sql)
            .bind(ctid)
            .execute(&mut *tx)
            .await
            .with_context(|| {
                format!(
                    "failed deleting row {ctid} from {}.{}",
                    table.schema, table.name
                )
            })?;
        deleted += result.rows_affected();
    }

    tx.commit()
        .await
        .context("failed to commit delete transaction")?;
    Ok(deleted)
}

fn query_returns_rows(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_ascii_lowercase();
    trimmed.starts_with("select")
        || trimmed.starts_with("values")
        || trimmed.starts_with("table")
        || trimmed.starts_with("show")
        || trimmed.starts_with("explain")
}

pub async fn run_query(pool: &PgPool, sql: &str, limit: usize) -> anyhow::Result<QueryOutput> {
    let normalized = sql.trim().trim_end_matches(';').trim();
    if normalized.is_empty() {
        anyhow::bail!("query is empty");
    }

    if query_returns_rows(normalized) {
        let wrapped = format!("SELECT * FROM ({normalized}) AS q LIMIT {limit}");
        let rows = sqlx::query(&wrapped)
            .fetch_all(pool)
            .await
            .context("failed to execute query")?;

        let Some(first_row) = rows.first() else {
            return Ok(QueryOutput::Rows {
                columns: vec![],
                rows: vec![],
                row_count: 0,
            });
        };

        let columns = first_row
            .columns()
            .iter()
            .map(|column| column.name().to_string())
            .collect::<Vec<_>>();

        let mut rendered_rows = Vec::with_capacity(rows.len());
        for row in rows {
            let rendered = (0..columns.len())
                .map(|index| row_cell_to_string(&row, index))
                .collect::<Vec<_>>();
            rendered_rows.push(rendered);
        }

        Ok(QueryOutput::Rows {
            columns,
            row_count: rendered_rows.len(),
            rows: rendered_rows,
        })
    } else {
        let result = sqlx::query(normalized)
            .execute(pool)
            .await
            .context("failed to execute command")?;

        Ok(QueryOutput::Command {
            rows_affected: result.rows_affected(),
        })
    }
}

fn row_cell_to_string(row: &sqlx::postgres::PgRow, index: usize) -> String {
    if let Ok(value) = row.try_get::<Option<String>, _>(index) {
        return value.unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i16>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i32>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<i64>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f32>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<sqlx::types::Uuid>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<sqlx::types::chrono::NaiveDate>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<sqlx::types::chrono::NaiveDateTime>, _>(index) {
        return value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) =
        row.try_get::<Option<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>>, _>(index)
    {
        return value
            .map(|v| v.to_rfc3339())
            .unwrap_or_else(|| "NULL".to_string());
    }
    if let Ok(value) = row.try_get::<Option<sqlx::types::Json<Value>>, _>(index) {
        return value
            .map(|v| v.0.to_string())
            .unwrap_or_else(|| "NULL".to_string());
    }

    "<unsupported>".to_string()
}

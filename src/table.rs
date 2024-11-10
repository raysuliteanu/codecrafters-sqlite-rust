use crate::page::DbRecord;

#[derive(Debug)]
pub struct TableInfo {
    pub(crate) name: String,
    table_name: String,
    root_page: i8,
    sql: String,
}

impl TableInfo {
    pub(crate) fn new(record: DbRecord) -> Result<Self, anyhow::Error> {
        let name = if let Some(c) = record.get(0) {
            match c {
                ColumnType::String(s) => String::from(s),
                _ => anyhow::bail!("column type mismatch"),
            }
        } else {
            anyhow::bail!("column type mismatch");
        };
        let table_name = if let Some(c) = record.get(1) {
            match c {
                ColumnType::String(s) => String::from(s),
                _ => anyhow::bail!("column type mismatch"),
            }
        } else {
            anyhow::bail!("column type mismatch");
        };
        let root_page = if let Some(c) = record.get(2) {
            match c {
                ColumnType::Int8(s) => *s,
                _ => anyhow::bail!("column type mismatch"),
            }
        } else {
            anyhow::bail!("column type mismatch");
        };
        let sql = if let Some(c) = record.get(3) {
            match c {
                ColumnType::String(s) => String::from(s),
                _ => anyhow::bail!("column type mismatch"),
            }
        } else {
            anyhow::bail!("column type mismatch");
        };

        Ok(TableInfo {
            name,
            table_name,
            root_page,
            sql,
        })
    }

    fn expect_column_type(col: ColumnType, expected_col: ColumnType) -> bool {
        col == expected_col
    }
}

#[derive(Debug, PartialEq)]
pub enum ColumnType {
    Null,
    Int8(i8),
    Int16(i16),
    Int24(i32),
    Int32(i32),
    Int48(i64),
    Int64(i64),
    Float(f64),
    False,
    True,
    Blob(Vec<u8>),
    String(String),
}

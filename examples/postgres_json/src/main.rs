use chrono::{NaiveDate, NaiveDateTime};
use postgres::{Client, NoTls, Row};
use sea_query::{ColumnDef, Iden, Order, PostgresQueryBuilder, Query, Table, PostgresDriver};

fn main() {
    let mut client = Client::connect("postgresql://sea:sea@localhost/query", NoTls).unwrap();

    // Schema

    let sql = [
        Table::drop()
            .table(Document::Table)
            .if_exists()
            .build(PostgresQueryBuilder),
        Table::create()
            .table(Document::Table)
            .if_not_exists()
            .col(ColumnDef::new(Document::Id).integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(Document::JsonField).json_binary())
            .col(ColumnDef::new(Document::Timestamp).timestamp())
            .build(PostgresQueryBuilder),
    ].join("; ");

    println!("{}", sql);
    let result = client.batch_execute(&sql).unwrap();
    println!("Create table document: {:?}\n", result);

    // Create
    let document = DocumentStruct {
        id: 1,
        json_field: serde_json::json! {{
            "a": 25.0,
            "b": "whatever",
            "c": {
                "another": "object",
                "bla": 1
            }
        }},
        timestamp: NaiveDate::from_ymd(2020, 1, 1).and_hms(2, 2, 2),
    };
    let (sql, values) = Query::insert()
        .into_table(Document::Table)
        .columns(vec![
            Document::JsonField, Document::Timestamp,
        ])
        .values_panic(vec![
            serde_json::to_value(document.json_field).unwrap().into(),
            document.timestamp.into(),
        ])
        .build(PostgresQueryBuilder);

    let result = client.execute(sql.as_str(), &values.as_params());
    println!("Insert into document: {:?}\n", result);

    // Read

    let (sql, values) = Query::select()
        .columns(vec![
            Document::Id, Document::JsonField, Document::Timestamp,
        ])
        .from(Document::Table)
        .order_by(Document::Id, Order::Desc)
        .limit(1)
        .build(PostgresQueryBuilder);

    let rows = client.query(sql.as_str(), &values.as_params()).unwrap();
    println!("Select one from document:");
    for row in rows.into_iter() {
        let item = DocumentStruct::from(row);
        println!("{:?}", item);
    }
    println!();
}

#[derive(Iden)]
enum Document {
    Table,
    Id,
    JsonField,
    Timestamp,
}

#[derive(Debug)]
struct DocumentStruct {
    id: i32,
    json_field: serde_json::Value,
    timestamp: NaiveDateTime,
}

impl From<Row> for DocumentStruct {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            json_field: row.get("json_field"),
            timestamp: row.get("timestamp"),
        }
    }
}

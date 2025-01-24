use rust_decimal::Decimal;
use rust_decimal_cql::DecimalCql;
use scylla::{DeserializeRow, SerializeRow, Session, SessionBuilder};

use std::env;
use std::str::FromStr;

#[derive(SerializeRow, DeserializeRow, Debug)]
struct Data {
    id: i64,
    decimal_1: DecimalCql,
    decimal_2: DecimalCql,
}

async fn create_schema(session: &Session) {
    session
        .query_unpaged("CREATE KEYSPACE IF NOT EXISTS example_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}", ())
        .await
        .expect("Failed to create keyspace");
    session
        .query_unpaged("CREATE TABLE IF NOT EXISTS example_keyspace.example_table (id BIGINT PRIMARY KEY, decimal_1 DECIMAL, decimal_2 DECIMAL)", ())
        .await
        .expect("Failed to create table");
}

async fn session() -> Session {
    let host = env::var("SCYLLA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let session: Session = SessionBuilder::new()
        .known_node(host)
        .build()
        .await
        .expect("Failed to connect to ScyllaDB");
    session
}

#[tokio::test]
async fn integration_test_positive_values() {
    let session = session().await;
    create_schema(&session).await;

    let dec_1 = Decimal::from_str("1.12345678956783").unwrap();
    let dec_2 = Decimal::from_str(".987654321123445445").unwrap();

    let data = Data {
        id: 1,
        decimal_1: dec_1.into(),
        decimal_2: dec_2.into(),
    };

    let val: Decimal = *data.decimal_1 / *data.decimal_2;
    assert_eq!(
        val,
        Decimal::from_str("1.1374999992810346914902557212").unwrap(),
    );

    session
        .query_unpaged(
            "INSERT INTO example_keyspace.example_table (id, decimal_1, decimal_2) VALUES (?, ?, ?)",
            data,
        )
        .await
        .expect("Failed to insert data into ScyllaDB");

    let result = session
        .query_unpaged(
            "SELECT id, decimal_1, decimal_2 FROM example_keyspace.example_table WHERE id = ?",
            (1i64,),
        )
        .await
        .expect("Failed to retrieve data from ScyllaDB");

    let rows = result.into_rows_result().expect("Failed to fetch rows");
    let retrieved_data: Data = rows.first_row().expect("No rows found");

    assert_eq!(
        *retrieved_data.decimal_1 / *retrieved_data.decimal_2,
        Decimal::from_str("1.1374999992810346914902557212").unwrap(),
    );
    assert_eq!(
        *retrieved_data.decimal_1,
        Decimal::from_str("1.12345678956783").unwrap()
    );
    assert_eq!(
        *retrieved_data.decimal_2,
        Decimal::from_str("0.987654321123445445").unwrap()
    );
}

#[tokio::test]
async fn integration_test_negative_values() {
    let session = session().await;
    create_schema(&session).await;

    let dec_1 = Decimal::from_str("-123.456").unwrap();
    let dec_2 = Decimal::from_str("-987.654").unwrap();

    let data = Data {
        id: 2,
        decimal_1: dec_1.into(),
        decimal_2: dec_2.into(),
    };

    session
        .query_unpaged(
            "INSERT INTO example_keyspace.example_table (id, decimal_1, decimal_2) VALUES (?, ?, ?)",
            data,
        )
        .await
        .expect("Failed to insert data into ScyllaDB");

    let result = session
        .query_unpaged(
            "SELECT id, decimal_1, decimal_2 FROM example_keyspace.example_table WHERE id = ?",
            (2i64,),
        )
        .await
        .expect("Failed to retrieve data");

    let rows = result.into_rows_result().unwrap();
    let retrieved_data: Data = rows.first_row().unwrap();

    assert_eq!(
        *retrieved_data.decimal_1,
        Decimal::from_str("-123.456").unwrap()
    );
    assert_eq!(
        *retrieved_data.decimal_2,
        Decimal::from_str("-987.654").unwrap()
    );

    assert_eq!(
        *retrieved_data.decimal_1 + *retrieved_data.decimal_2,
        Decimal::from_str("-1111.11").unwrap()
    );
}

#[tokio::test]
async fn integration_test_zero_values() {
    let session = session().await;
    create_schema(&session).await;

    let dec_1 = Decimal::from_str("0.0").unwrap();
    let dec_2 = Decimal::from_str("123.456").unwrap();

    let data = Data {
        id: 3,
        decimal_1: dec_1.into(),
        decimal_2: dec_2.into(),
    };

    session
        .query_unpaged(
            "INSERT INTO example_keyspace.example_table (id, decimal_1, decimal_2) VALUES (?, ?, ?)",
            data,
        )
        .await
        .expect("Failed to insert data into ScyllaDB");

    let result = session
        .query_unpaged(
            "SELECT id, decimal_1, decimal_2 FROM example_keyspace.example_table WHERE id = ?",
            (3i64,),
        )
        .await
        .expect("Failed to retrieve data");

    let rows = result.into_rows_result().unwrap();
    let retrieved_data: Data = rows.first_row().unwrap();

    assert_eq!(*retrieved_data.decimal_1, Decimal::from_str("0.0").unwrap());
    assert_eq!(
        *retrieved_data.decimal_2,
        Decimal::from_str("123.456").unwrap()
    );

    let val: Decimal = *retrieved_data.decimal_1 * *retrieved_data.decimal_2;
    assert_eq!(val, Decimal::from_str("0.0").unwrap());
}

#[tokio::test]
async fn example() {
    let session = session().await;
    create_schema(&session).await;
    // Create Decimals
    let decimal_1 = Decimal::from_str("3.1234567890987654321").unwrap();
    let decimal_2 = Decimal::from_str("100.0987654321234567890").unwrap();
    // Create the data struct, wrapping Decimal
    let data: Data = Data {
        id: 1,
        decimal_1: decimal_1.into(),
        decimal_2: decimal_2.into(),
    };
    // Access the Decimal by dereferencing
    let value: Decimal = *data.decimal_1 / *data.decimal_2;

    // Store the struct, no boilerplate code needed, just pass the struct
    session.query_unpaged(
        "INSERT INTO example_keyspace.example_table (id, decimal_1, decimal_2) VALUES (?, ?, ?)",
        data, )
        .await
        .expect("Failed to insert data into ScyllaDB");

    // Retrieve the row
    let rows = session
        .query_unpaged(
            "SELECT id, decimal_1, decimal_2 FROM example_keyspace.example_table WHERE id = ?",
            (1i64,),
        )
        .await
        .expect("Failed to retrieve data from ScyllaDB")
        .into_rows_result()
        .expect("Failed to fetch rows");

    // No boilerplate code needed, just use type annotation
    let data: Data = rows.first_row().expect("No rows found");
    // Access the retrieved Decimal by dereferencing
    let value: Decimal = *data.decimal_1 * *data.decimal_2;

    // Validate that the retrieved data matches the inserted data
    assert_eq!(
        *data.decimal_1,
        Decimal::from_str("3.1234567890987654321").unwrap()
    );
    assert_eq!(
        *data.decimal_2,
        Decimal::from_str("100.0987654321234567890").unwrap()
    );
}

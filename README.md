# DecimalCql &nbsp; [![Build Status](https://github.com/erik404/rust_decimal_cql/actions/workflows/unit_integration_test.yml/badge.svg)](https://github.com/erik404/rust_decimal_cql/actions/)

[rust_decimal_cql::DecimalCql](https://crates.io/crates/rust_decimal_cql) is a wrapper
around [rust_decimal::Decimal](https://crates.io/crates/rust_decimal) that implements
the [DeserializeValue](https://docs.rs/scylla/0.15.1/scylla/deserialize/trait.DeserializeValue.html)
and [SerializeValue](https://docs.rs/scylla/0.15.1/scylla/serialize/value/trait.SerializeValue.html) traits from
the [ScyllaDB Rust Driver](https://crates.io/crates/scylla). This enables seamless integration with ScyllaDB's
native [DECIMAL](https://docs.rs/scylla/0.15.1/scylla/transport/topology/enum.NativeType.html#variant.Decimal) column type.
`DecimalCql` is fully compatible with the [DeserializeRow](https://docs.rs/scylla/0.15.1/scylla/deserialize/trait.DeserializeRow.html)
and [SerializeRow](https://docs.rs/scylla/0.15.1/scylla/serialize/row/trait.SerializeRow.html) traits, making it safe and effortless to use
within structs for storing and retrieving data directly from ScyllaDB.

## Why Use DecimalCql?

- **Reliable Serialization and Deserialization:** Converts `Decimal` to and from ScyllaDB's native `DECIMAL`.
- **Type Safety:** Ensures that your `Decimal` values retain precision and accuracy during database operations.
- **Struct Compatibility:** Designed for use in Rust structs, making it easy to work with ScyllaDB rows.

## Example

```rust

// Demonstrates the usage of DecimalCql for storing and retrieving Decimal values in ScyllaDB.

use rust_decimal::Decimal;
use rust_decimal_cql::DecimalCql;
use scylla::Session;
use scylla::query::Query;
use scylla::{DeserializeRow, SerializeRow, QueryRowsResult};

#[derive(SerializeRow, DeserializeRow, Debug)]
struct Data {
    id: i64,
    decimal_1: DecimalCql,
    decimal_2: DecimalCql,
}

async fn example(session: Session) {
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
```

---

Contributions are welcome! Feel free to open an issue or submit a pull request with improvements or new features.

---

## License

This library is licensed under the [MIT License](LICENSE).


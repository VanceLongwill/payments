## Payments

Payments system in Rust

- Uses a lightweight DDD (domain driven design) approach. The business logic is
separated into the relevant aggregates and ports in out of the application are contained
within `main.rs`.

- Makes use of Rust's type system to ensure correctness (see the `TransactionKind` enum)

- Aims to be easily extensible and loosely coupled. The CSV format can be easily changed.


## TODO:

- Add more tests
- Limit the serialized `Decimal` precision to 4 decimal places

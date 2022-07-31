## Payments

Payments system in Rust

- Uses a lightweight DDD (domain driven design) approach. The business logic is
separated into the relevant aggregates and ports in out of the application are contained
within `main.rs`.

- Makes use of Rust's type system to ensure correctness (see the `TransactionKind` enum)

- Aims to be easily extensible and loosely coupled. The CSV format can be easily changed.


## Running the project

```sh
$ cargo run -- example.csv
```

With debug logs:
```sh
$ RUST_LOG=debug cargo run -- example.csv
Jul 31 12:02:03.857 DEBUG payments: Processed transaction tx=1 client=1
Jul 31 12:02:03.858 DEBUG payments: Processed transaction tx=2 client=2
Jul 31 12:02:03.858 DEBUG payments: Processed transaction tx=3 client=1
Jul 31 12:02:03.858 DEBUG payments: Processed transaction tx=4 client=1
Jul 31 12:02:03.858 DEBUG payments: Unable to process transaction error="insufficient funds" tx=5 client=2
Jul 31 12:02:03.858 DEBUG payments: Processed transaction tx=6 client=1
Jul 31 12:02:03.858 DEBUG payments: Processed transaction tx=2 client=2
Jul 31 12:02:03.858 DEBUG payments: Processed transaction tx=1 client=1
Jul 31 12:02:03.858 DEBUG payments: Processed transaction tx=1 client=1
Jul 31 12:02:03.858 DEBUG payments: Unable to process transaction error="unable to move transaction from ChargeBack to Deposit { amount: 1 }" tx=1 client=1
```


## TODO:

- Add more tests
- Limit the serialized `Decimal` precision to 4 decimal places

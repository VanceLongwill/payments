use rust_decimal::prelude::*;

use serde::Deserialize;

#[derive(PartialEq, Debug, Clone)]
enum LockedStatus {
    Locked,
    Unlocked,
}

impl Serialize for LockedStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(match self {
            LockedStatus::Locked => true,
            LockedStatus::Unlocked => false,
        })
    }
}

#[derive(Serialize)]
struct Statement {
    available: Decimal,
    held: Decimal,
    locked: LockedStatus,
}

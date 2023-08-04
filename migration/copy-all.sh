#!/usr/bin/env bash

cargo run --bin migration copy-database --from MaccasAuditData         --to MaccasApi-LastRefresh &
cargo run --bin migration copy-database --from MaccasApiCache-v2       --to MaccasApi-Deals &
cargo run --bin migration copy-database --from MaccasApiUserConfig-v2  --to MaccasApi-UserConfig &
cargo run --bin migration copy-database --from MaccasApiDb             --to MaccasApi-Tokens &
cargo run --bin migration copy-database --from MaccasApiCache          --to MaccasApi-Accounts &
cargo run --bin migration copy-database --from MaccasApiOfferId        --to MaccasApi-LockedOffers &
cargo run --bin migration copy-database --from MaccasApiPointDb        --to MaccasApi-Points &
cargo run --bin migration copy-database --from MaccasRefreshTrackingDb --to MaccasApi-RefreshTracking &
cargo run --bin migration copy-database --from MaccasAudit             --to MaccasApi-Audit &
cargo run --bin migration copy-database --from MaccasUserAccounts      --to MaccasApi-UserAccounts &

wait "$(jobs -p)"

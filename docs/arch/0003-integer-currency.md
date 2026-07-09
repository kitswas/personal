# ADR 0003 — Integer Representation of Currency Amounts

- **Date:** 2026-07-08
- **Status:** Accepted

## Context

Financial applications must represent money without rounding error. IEEE 754
double-precision floating-point (the default numeric type in JavaScript, SQLite
`REAL`, and naïve Rust) introduces representational errors:

```
0.1 + 0.2 = 0.30000000000000004   // IEEE 754
```

For accounting, even a single sub-unit rounding error can cause a transaction
to fail the double-entry balancing check (`SUM(postings.amount) = 0`), silently
corrupt running balances, or produce incorrect tax totals.

Alternatives considered:

| Approach                              | Pros                                     | Cons                                      |
| ------------------------------------- | ---------------------------------------- | ----------------------------------------- |
| `f64` / SQLite `REAL`                 | Native in JS and SQL                     | Rounding errors; cannot guarantee SUM = 0 |
| `Decimal` crate (arbitrary precision) | Exact; handles multi-currency arithmetic | Heavier; not natively stored in SQLite    |
| `INTEGER` (smallest unit)             | Exact; native SQLite type; zero overhead | Requires explicit scaling in UI layer     |
| `TEXT` (string decimals)              | Exact representation                     | No native arithmetic; slow aggregation    |

## Decision

All monetary amounts are stored as `INTEGER` in SQLite, representing the
**smallest indivisible unit** of the currency.

For INR (the default): 1 rupee = 100 paise → store amounts in paise.
For USD: 1 dollar = 100 cents → store in cents.
For JPY (no sub-unit): store in yen directly.

The `commodity` column on `postings` records the currency so the UI layer
knows the correct scale factor to apply when displaying amounts.

```sql
amount INTEGER NOT NULL   -- paise (for INR), cents (for USD), etc.
```

The Rust backend always works with `i64`. The frontend converts for display:

```typescript
// Pure function — no mutation
const formatAmount = (paise: number, commodity: string): string => { ... }
```

The double-entry invariant is checked in integer arithmetic:

```rust
let sum: i64 = postings.iter().map(|p| p.amount).sum();
if sum != 0 {
    return Err(AppError::UnbalancedTransaction { sum });
}
```

## Consequences

- **Good:** Exact arithmetic. No rounding errors. The balancing invariant is
  mathematically provable.
- **Good:** SQLite `SUM()` over `INTEGER` columns is exact.
- **Good:** `i64` has sufficient range: 2^63 - 1 paise ≈ 92 quadrillion rupees,
  far exceeding any realistic financial value.
- **Trade-off:** The UI must scale amounts by the commodity's sub-unit factor.
  This is a single pure formatting function; it is not complex.
- **Constraint:** V1 is single-currency per posting. Multi-currency postings
  (e.g., forex transactions) will require a future ADR and schema change.

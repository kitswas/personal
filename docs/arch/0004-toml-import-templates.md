# ADR 0004 — TOML for Import Templates

- **Date:** 2026-07-08
- **Status:** Accepted

## Context

The import pipeline uses declarative templates to map bank-statement columns
to double-entry postings. Templates must express:

- Column-to-field mappings (date, payee, amount)
- Regular expressions for extracting data from free-text fields (e.g., UPI strings)
- Multi-leg posting rules (arrays of debit/credit legs)
- Inline comments for human maintainers

Formats considered:

| Format             | Regex ergonomics                             | Comments      | Arrays of tables   | Verdict                                  |
| ------------------ | -------------------------------------------- | ------------- | ------------------ | ---------------------------------------- |
| JSON               | Poor — `\\d+` must be `\\\\d+`               | Not supported | Verbose            | Rejected                                 |
| YAML               | Poor — special chars need quoting            | Supported     | Supported          | Rejected                                 |
| TOML               | **Excellent** — `'\\d+'` is a literal string | Supported     | `[[table]]` syntax | **Chosen**                               |
| Lua / WASM scripts | Full programmability                         | —             | —                  | Rejected — arbitrary code execution risk |

The decisive factor is TOML's **literal string** syntax (single quotes).
A regex like `UPI/(?:DR|CR)/\d+/([^/]+)` requires no escaping in TOML:

```toml
# TOML literal string — what you see is what the regex engine gets
regex = 'UPI/(?:DR|CR)/\d+/([^/]+)'
```

The same pattern in JSON:

```json
"regex": "UPI/(?:DR|CR)/\\d+/([^/]+)"
```

For complex financial regex patterns (UPI references, NEFT/RTGS codes, IFSC
strings), the JSON escaping burden is error-prone and hard to review.

TOML's `[[array of tables]]` syntax also maps naturally to multi-leg postings:

```toml
[[postings_rules.legs]]
description = "Gross Income"
amount_column = "Amount Paid/Credited"
direction = "credit"
default_account = "Income:Salary"

[[postings_rules.legs]]
description = "Tax Deducted"
amount_column = "Tax Deducted"
direction = "debit"
default_account = "Asset:Taxes:TDS_Receivable"
```

Lua or WASM scripts would allow full programmability but introduce an arbitrary
code execution surface — unacceptable under the zero-trust threat model.

## Decision

Import templates are TOML files, parsed by the `toml` crate in Rust.
They are bundled with the binary via `include_bytes!` and can
also be loaded from a user-specified directory.

All regex patterns in templates are compiled with `regex::RegexBuilder`
and a `size_limit` to prevent catastrophic backtracking (termination guarantee).

## Consequences

- **Good:** Literal string syntax eliminates the regex-escape error class.
- **Good:** Comments make templates self-documenting for non-technical users
  who maintain their own bank templates.
- **Good:** `[[array of tables]]` cleanly expresses multi-leg postings.
- **Good:** `toml` crate is mature, widely used, and has no transitive
  cryptographic dependencies.
- **Trade-off:** TOML is less familiar than JSON to most users. Mitigation:
  pre-bundled templates cover the common Indian bank formats; most users will
  never write their own.
- **Constraint:** Template files must be valid UTF-8. Files with Windows-1252
  or other encodings must be converted before use.

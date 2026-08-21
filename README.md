# Claim Settlement Service — Backend Coding Challenge

Welcome! This challenge mirrors a real slice of our domain: bike leasing contracts,
an insurer that reports claims, and a backend service that has to make sense of both.

Please read this document completely before writing code. Everything you need is in
here and in `data/`.

---

## Scenario

Employees lease bikes through their employer. Each individual leasing contract
(**ELV**, e.g. `ELV-100234`) originates from a sale order (**KAU**, e.g. `KAU-003945`)
and covers one bike for a fixed term. The bikes are insured; the insurer reports
**claims** (theft, damage, …) that reference the sale order number.

Our ERP produces nightly XML exports. You receive three of them in `data/`:

| File | Contents |
|---|---|
| `data/contracts_export.xml` | Leasing contracts (one-time snapshot) |
| `data/claims_batch_1.xml` | Claims from the insurer feed, day 1 |
| `data/claims_batch_2.xml` | Claims from the insurer feed, day 2 |

The exports are **production-like**. Real exports contain surprises; so do these.
Discovering and handling them sensibly is part of the task; document what you find
in `NOTES.md`.

## Your task

Build a small monolithic backend service that:

1. **Ingests** the XML exports into PostgreSQL.
2. **Decides claims** by applying the settlement rules below.
3. Exposes a small **HTTP API** to browse contracts/claims and trigger decisions.

Frontend is optional and is treated as a bonus. We review your work through the
API and the database.

---

## 1. Ingestion

- Ingestion must be **repeatable**: ingesting the same file twice must not create
  duplicates. Claims are identified by `external_id`, contracts by `name`.
- `claims_batch_2.xml` arrives *after* batch 1 and may contain records you have
  already seen; the newer data wins.
- How ingestion is triggered (API endpoint, CLI command, startup hook, …) is your
  design decision. Document it in `NOTES.md` and make it easy for us to run.
- Claims reference contracts via `sale_order_number`. What you do with records that
  don't line up cleanly is also your design decision: decide, implement, and
  document it.

## 2. Settlement rules

A **decision** is the outcome of processing a claim. Decisions are persisted.

### Decision statuses

- `APPROVED`: carries a `payout_amount`
- `REJECTED`: carries a machine-readable `reason`
- `MANUAL_REVIEW`: carries a `reason`, for claims your rules cannot safely decide

### Preconditions (apply to every claim type)

A claim can only be decided if **all** of these hold; otherwise it is `REJECTED`
(or `MANUAL_REVIEW` where you judge that more appropriate):

1. The referenced contract exists.
2. The contract is in state `active` **at the time the decision is made**.
3. The claim's `incident_date` lies within the leasing period
   (`start_leasing` ≤ incident ≤ `end_leasing`).
4. The claim is in a decidable state (see the state table below).

### Per-type rules

| Type | Extra requirement | Payout | Side effect |
|---|---|---|---|
| `THEFT` | police report number present, otherwise `REJECTED` | residual value | — |
| `PARTIAL_DAMAGE` | `repair_cost` must exceed the €50 deductible, otherwise `REJECTED` | `repair_cost − 50.00` | — |
| `TOTAL_DAMAGE` | — | `residual value − 150.00` (not below 0) | contract state → `done` (ends early) |

The feed may contain claim types not listed here. Your service must not crash on
them; how you handle them is your design decision.

### Residual value

```
residual_value = net_value × remaining_months / total_months
```

- `total_months` = calendar months **fully contained** in
  `[start_leasing, end_leasing]`
- `remaining_months` = calendar months **fully contained** in
  `[incident_date, end_leasing]`
- Round money **half up** to cents, and use an exact decimal representation for
  money; binary floating-point arithmetic introduces rounding errors.

**Worked example**: contract `ELV-100234` has `net_value` 3499.00 and leasing
period 2024-09-01 → 2027-08-31. For a theft on **2026-03-23**:

- `total_months` = Sep 2024 … Aug 2027 = **36**
- `remaining_months` = Apr 2026 … Aug 2027 = **17** (March 2026 is not fully
  contained because the incident falls on the 23rd)
- `residual_value` = 3499.00 × 17 / 36 = **1652.31**

Claim `778120` in batch 1 is exactly this case; use it as a sanity check.

### Claim states

The insurer feed reports claim states as German display strings. Store the
canonical value:

| In the feed | Canonical | Decidable? |
|---|---|---|
| `Schaden gemeldet / prüfen` | `INVESTIGATION` | yes |
| `Unterlagen angefordert` | `AWAITING_DOCUMENTS` | yes |
| `reguliert`, `bezahlt` | `SETTLED` | no (final) |
| `abgelehnt` | `NOT_SETTLED` | no (final) |
| `gegenstandslos` | `INVALID` | no (final) |

Attempting to decide a claim in a final state must fail with a clear error
(HTTP `409` is a good fit).

### Decision semantics

- Deciding is **idempotent**: requesting a decision for an already-decided claim
  returns the stored decision unchanged; side effects must not run twice.
- A decision records at minimum: status, payout amount (if any), reason (if any),
  and when it was made.

## 3. API

The exact shape is yours to design; it must at least support:

```
GET  /contracts                     list contracts (filter by state is a plus)
GET  /contracts/{name}              contract detail incl. its claims
GET  /claims                        list claims (filter by state/type is a plus)
POST /claims/{external_id}/decision decide a claim, return the decision
```

Return meaningful HTTP status codes. Update `requests.http` (or provide a curl
script) so we can replay your main flows in under a minute.

---

## Tech constraints

- **Language**: Python (FastAPI or Django) or Kotlin/Java (Spring Boot), the
  languages of our stack. If you would like to use something else, please ask
  us first.
- **Database**: the provided PostgreSQL (see `docker-compose.yml`). Use an ORM or
  plain SQL; either is fine.
- **Runs as a monolith**: `docker compose up --build` from a clean checkout must
  bring up your service on a local port. Port 8000 is preconfigured as an example;
  feel free to change the mapping. Replace `app/Dockerfile` (the placeholder just
  prints a message).
- The XML files are mounted read-only at `/data` inside the app container.

Inspecting the database:

```
docker compose exec db psql -U challenge -d challenge
```

## Time

We expect **about 4 hours** of focused work. When you reach that limit, stop; an
honest cut with clear notes is worth more to us than extra hours. You will most
likely not finish everything. Deciding what to build first and what to leave out
is part of the task, so record your priorities and cuts in `NOTES.md`.

Whatever you build, test it. The settlement rules are pure logic and inexpensive
to test; a smaller, tested scope beats a larger, untested one.

**Commit as you go.** We read your git history; a single "final version" commit
tells us very little about how you work.

## AI tools

AI tools are allowed and expected; use what you would normally use on the job.
Disclose in `NOTES.md` which tools you used and for what, including at least one
place where you rejected or reworked their output. In the follow-up session we
will work on your code together, so make sure you can explain every line you
submit.

## Deliverables

- [ ] The repo as a zip **including `.git`** (commit history intact)
- [ ] `docker compose up --build` works from a clean checkout
- [ ] Source + tests under `app/`
- [ ] `NOTES.md` filled in (decisions, data findings, AI usage, cuts)
- [ ] `requests.http` (or equivalent) demonstrating your API

## What we look at

- Correctness against the provided data
- Design and structure: centralized rules and clear boundaries. In the follow-up
  session we will extend your code together, so design for the next claim type,
  not only the three above.
- Tests that run with a single command
- Communication: git history, `NOTES.md`, and a clear API

Good luck! We look forward to your submission.

# Roadmap

Two tracks with independent progress: **patchwaste** (free open-core CLI on GitHub) and **patchwaste-cloud** (private monetizable SaaS).

Product boundary rule: if a feature requires shared multi-user state, centralized governance, engine-specific diagnosis, or managed operations, it belongs in patchwaste-cloud.

---

## patchwaste (open-core, free tier)

### Shipped

- [x] SteamPipe BuildOutput log parser (3 regex patterns)
- [x] Core metrics: waste_ratio, delta_efficiency, confidence levels
- [x] Rule R1: HIGH_WASTE_RATIO (fires at >= 50%)
- [x] Rule R2: LARGE_TOP_OFFENDER (fires at >= 100 MB)
- [x] Baseline comparison (regression_ratio from prior report.json)
- [x] Budget gating (pass/fail signal from regression_ratio threshold)
- [x] JSON report output (versioned schema)
- [x] Markdown report output
- [x] CLI `analyze` subcommand with --input, --baseline, --budget-ratio, --strict, --out
- [x] Exit codes: 0 pass, 2 budget fail, 1 error
- [x] NO_COLOR env var support
- [x] GitHub Actions CI (fmt, clippy, test)
- [x] Integration + snapshot tests (92% line coverage)
- [x] Governance docs (LICENSE, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, TRADEMARK)

### Remaining

- [x] `patchwaste.toml` config file (app_id, depot_ids, branches, per-depot budgets)
- [x] Per-depot metrics (currently only totals; design calls for `per_depot[]`)
- [x] `build_metadata` in report JSON (sha, branch, build_id)
- [x] JUnit XML output (`--output-format junit`)
- [x] GitLab CI template
- [x] Fuzz and property-based tests
- [x] Remove unused `thiserror` dependency

---

## patchwaste-cloud (private SaaS, monetizable)

### Shipped

- [x] Cloud positioning and concept docs
- [x] Monetisation model and tiering design (Free / Pro / Studio)
- [x] Open-core scope boundary definition
- [x] Cloud FAQ and onboarding narrative

### Remaining — Diagnostics (paid differentiator)

- [ ] Rule R3: TOC diffusion signature detection
- [ ] Rule R4: Cross-asset compression detection
- [ ] Engine-specific diagnostics (Unreal/Unity pack pattern heuristics)
- [ ] `links_to_runbook` field on findings (actionable fix guidance)

### Remaining — Workflow integrations

- [ ] PR comment formatter
- [ ] GitHub App / PR annotation service

### Remaining — Platform

- [ ] Managed baseline/history API
- [ ] Org policy engine and team roles
- [ ] Dashboard for trend analysis and release sign-off
- [ ] License/entitlement system (signed JWT tokens for CI)
- [ ] Payment integration (MoR or Stripe)

### Remaining — Enterprise

- [ ] SSO integration
- [ ] Audit logging
- [ ] Support SLA and private onboarding

---

## Signals used to prioritize patchwaste-cloud acceleration

- Number of active repos repeatedly using open-core CLI
- Frequency of requests for shared policy/history workflows
- Trust level in open-core metrics for gatekeeping decisions

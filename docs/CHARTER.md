# Patchwaste — Project Charter

## Vision & USP

Patchwaste measures SteamPipe patch size inefficiency as a CI gate signal, alerting game developers when a build will produce an oversized player download.

## Goals

1. Achieve ≥95% accuracy in patch size waste estimation for UE5 pak-aligned builds, validated against real depot manifests.
2. Earn 25 GitHub stars or verified CI integrations within 60 days of v1 announcement.
3. Ship patchwaste-cloud Pro tier (baselines + PR annotations + Paddle billing) within 90 days of v1 ship.
4. Land one paying studio customer within 6 months of cloud launch.
5. Keep the false-positive rate for CI gate failures below 5% in real-world usage.

## Definition of Done

v1 is shipped when: a GitHub Actions workflow using the published action produces deterministic pass/fail CI results with a configurable waste-ratio threshold, a Markdown report with engine-specific remediation guidance, and baseline regression tracking — verified end-to-end against a synthetic UE5 depot fixture.

## Business Justification

Eight years of documented complaints and zero tooling solutions. Studios discover patch bloat after it ships to players — the Enshrouded incident (299 KB change triggering a 30.6 GB download) is the canonical example, and it is not isolated. The open-core CLI builds developer trust first; the cloud tier captures recurring B2B revenue from studios that need policy enforcement, PR automation, and multi-branch history. No funded competitor occupies this CI gate position.

## Monetization Path

1. Free open-core CLI earns installs and trust. Viral when studios share CI configs.
2. patchwaste-cloud Pro (£19–£49/mo per title): managed baselines, PR annotations, 90-day regression history.
3. Studio plan (£99–£299/mo org): multi-title, multi-branch policies, audit log, SSO (later).
4. Billing via Paddle (MoR) to keep VAT/tax handling off the critical path early.

## Quantified Impact

- Addressable market: ~7,000 active Steam titles with frequent updates; UE5 + GDK studios are the first beachhead.
- Revenue target: £10k ARR within 12 months of cloud launch.
- CDN/player cost avoided: a 30 GB unnecessary patch costs a studio roughly £500–£2,000 in Valve bandwidth + player goodwill damage. The tool pays for itself in one prevented incident.
- Organic distribution: CI configs are public in game repos — every visible workflow is a free ad.

## Target Audience

UE5 studios shipping Steam updates more than once a month, 5–50 person teams, using GitHub Actions or GitLab CI. Primary persona: the build/release engineer at a mid-size indie or AA studio who owns the CI pipeline and cares about player patch size. Secondary: open-core contributors from the game tooling community who self-select via GitHub.

## Time Estimates

- **v0.1 (MVP):** Done — open-core CLI with waste ratio analysis, CI gate, JSON/Markdown output, baseline regression.
- **v1.0 (Launchable):** Done — `patchwaste.toml` config, JUnit output, GitLab CI template, contribution hardening, v1 release checklist.
- **v2.0 (Growth):** patchwaste-cloud: managed baselines, PR annotations, policy engine, Paddle billing, GitHub App/Marketplace listing.

## Rubric

- [x] I can explain the USP in one sentence
- [x] At least one goal is measurable within 30 days
- [x] Definition of done is concrete enough to test
- [x] Business justification doesn't rely on "it would be cool"
- [x] Monetization path has at least one realistic step
- [x] Target audience is a real group I can reach
- [x] Time estimates account for integration and testing

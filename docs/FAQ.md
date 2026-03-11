# FAQ

## Is patchwaste open source?

Yes. This repository is Apache-2.0 licensed. See `LICENSE`.

## Can companies use this commercially?

Yes, Apache-2.0 allows commercial use of the open-core code.

## What is the business model then?

Open core is the local CLI/parser/reporting engine.
Commercial value is planned in hosted multi-user workflow: managed history, org policies, dashboards, SSO/audit, and support SLAs.

## Why keep the core public?

Public code improves trust, adoption, and integration speed.
The moat is workflow/governance/support quality, not hiding a metric formula.

## What is in scope for open-core feature requests?

Local-first capabilities: parsing quality, report clarity, baseline comparison, CLI usability, CI ergonomics.

## What is out of scope for open-core requests?

Hosted dashboard, org accounts, centralized policy administration, enterprise controls, and SLA commitments.

## Can I contribute?

Yes. See `CONTRIBUTING.md` and sign commits with DCO (`git commit -s ...`).

## Can forks use the patchwaste name/logo?

Not by default. Code is open under Apache-2.0, but trademark use is governed by `TRADEMARK.md`.

## Where do I report security issues?

Use the private channel in `SECURITY.md`.

## What is patchwaste-cloud?

patchwaste-cloud is the private SaaS layer that adds managed baselines and history,
policy governance, dashboards, alerts, and CI/PR integrations on top of the open-core CLI.

## What data is sent?

Only metadata and report summaries are sent; build artifacts stay local.
Retention is configurable per org/project.

## How does pricing work?

patchwaste-cloud is a SaaS subscription priced per org and/or title.
No usage pricing or per-seat fees are required for the initial offering.

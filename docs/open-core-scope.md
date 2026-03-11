# Open Core Scope

This project uses an Apache-2.0 open-core model.

## Who v0 targets

- Unreal teams using SteamPipe preview output
- CI owners who need a fast regression signal for patch waste

## Included in open core (public GitHub)

- SteamPipe/BuildOutput parsing and metric extraction
- Local CLI workflows (`analyze`, baseline comparison, budget gate)
- Stable JSON/markdown/text reporting formats
- Tests and fixtures needed to validate parser correctness
- Local/CI usage guidance

## Explicitly not included in open core

- Hosted dashboard and long-term managed history
- Team/org account model and access controls
- Managed policy distribution and centralized budget governance
- SSO, audit logs, enterprise compliance controls
- Commercial SLAs and private onboarding services

## Product boundary rule

If a feature requires shared multi-user state, centralized governance, or managed operations, it belongs in the patchwaste-cloud.

## Why this split

Open core stays minimal but useful for single-team local workflows.
patchwaste-cloud value focuses on workflow integration, governance, and operational support.

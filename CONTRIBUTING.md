# Contributing

Thanks for your interest in improving `patchwaste`.

## Development flow

1. Open an issue (or confirm an existing one) before major changes.
2. Create a focused branch.
3. Add/adjust tests with your change.
4. Bootstrap your local dev environment (one-time setup):

```bash
./scripts/bootstrap-dev.sh
```

5. Run local checks (same as CI, also enforced by hooks):

```bash
./scripts/verify.sh
```

6. Submit a PR with clear rationale and test evidence.

## Sign-off requirement (DCO)

By contributing, you certify the Developer Certificate of Origin (DCO) for your commit by adding a sign-off line:

```bash
git commit -s -m "feat: your change"
```

This appends:

`Signed-off-by: Your Name <you@example.com>`

## Licensing

Contributions are accepted under the repository's Apache-2.0 license.

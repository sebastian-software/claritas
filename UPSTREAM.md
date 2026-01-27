# Upstream Dependencies

This project ports the following libraries to Rust:

## Mozilla Readability

- **Repository**: https://github.com/mozilla/readability
- **License**: Apache-2.0
- **Location**: `upstream/readability/`
- **Method**: Git subtree

## Defuddle

- **Repository**: https://github.com/kepano/defuddle
- **License**: MIT
- **Location**: `upstream/defuddle/`
- **Method**: Git subtree

## Updating Upstream

To update the Readability subtree:

```bash
git subtree pull --prefix=upstream/readability https://github.com/mozilla/readability.git main --squash
```

To update the Defuddle subtree:

```bash
git subtree pull --prefix=upstream/defuddle https://github.com/kepano/defuddle.git main --squash
```

## Fixtures

Test fixtures are mirrored from upstream to `fixtures/`:

- `fixtures/readability/` - from `upstream/readability/test/test-pages/`
- `fixtures/defuddle/` - from `upstream/defuddle/tests/`

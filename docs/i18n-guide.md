# ZeroClaw i18n Completion Guide

This guide defines how to keep multilingual documentation complete and consistent when docs change.

## Scope

Use this guide when a PR touches any user-facing docs navigation, shared docs wording, or runtime-contract references.

Primary docs surfaces:

- Root READMEs: `README.md`, `README.<locale>.md`
- Docs hubs: `docs/README.md`, `docs/i18n/<locale>/README.md`
- Unified TOC: `docs/SUMMARY.md`
- i18n index and coverage: `docs/i18n/README.md`, `docs/i18n-coverage.md`

Supported locales:

- `en` (source of truth)
- `zh-CN`, `ja`, `ru`, `fr` (hub-level localized + i18n scaffold)
- `vi`, `el` (full localized trees)

## Canonical Layout

Required structure:

- Root language landing: `README.<locale>.md`
- Canonical localized docs hub: `docs/i18n/<locale>/README.md`
- Canonical localized summary: `docs/i18n/<locale>/SUMMARY.md`

Compatibility shims may exist at docs root (for example `docs/README.zh-CN.md`, `docs/SUMMARY.zh-CN.md`) and must remain aligned when touched.

## Trigger Matrix

Use this matrix to decide required i18n follow-through in the same PR.

| Change type | Required i18n follow-through |
|---|---|
| Root README language switch line changed | Update language switch line in all root `README*.md` files |
| Docs hub language links changed | Update localized hub links in `docs/README.md` and every `docs/README*.md` / `docs/i18n/*/README.md` with an "Other languages" section |
| Unified TOC language entry changed | Update `docs/SUMMARY.md` and every localized `docs/SUMMARY*.md` / `docs/i18n/*/SUMMARY.md` language-entry section |
| Runtime-contract docs changed (`commands`, `providers`, `channels`, `config`, `runbook`, `troubleshooting`, `one-click`) | Update `vi` and `el` equivalents where present; for other locales, update hub links/status notes if full translation is not available |
| Locale added/removed/renamed | Update root READMEs, docs hubs, summaries, `docs/i18n/README.md`, and `docs/i18n-coverage.md` |

## Completion Checklist (Mandatory)

Before merge, verify all items:

1. Locale navigation parity
- Root language switch line includes all supported locales.
- Docs hubs include all supported locales.
- Summary language entry includes all supported locales.

2. Canonical path consistency
- Non-English hubs point to `docs/i18n/<locale>/README.md`.
- Non-English summaries point to `docs/i18n/<locale>/SUMMARY.md`.
- Compatibility shims do not contradict canonical entries.

3. Runtime-contract parity
- If runtime-contract docs changed, sync `vi` and `el` localized files when equivalents exist.
- If `zh-CN`/`ja`/`ru`/`fr` cannot be fully translated in the same PR, update hubs/coverage status and create explicit follow-up tracking.

4. Coverage metadata
- Update `docs/i18n-coverage.md` if support status, canonical path, or coverage level changed.
- Keep date stamps current for changed localized hubs/summaries.

5. Link integrity
- Run markdown/link checks (or equivalent local relative-link existence check) on changed docs.

## Deferred Translation Policy

If full localization cannot be completed in the same PR:

- Keep navigation parity complete (never leave locale links partially updated).
- Add explicit deferral note in PR description with owner and follow-up issue/PR.
- Update `docs/i18n-coverage.md` to reflect temporary status.

Do not silently defer user-facing language parity changes.

## Agent Workflow Contract

When an agent touches docs IA or shared docs wording, the agent must:

1. Apply this guide and complete i18n follow-through in the same PR.
2. Update `docs/i18n-coverage.md` and `docs/i18n/README.md` when locale topology changes.
3. Include i18n completion notes in PR summary (what was synced, what was deferred, why).

## Gap Tracking

- Track remaining locale-depth gaps in [i18n-gap-backlog.md](i18n-gap-backlog.md).
- Update [i18n-coverage.md](i18n-coverage.md) and backlog counts after each localization wave.

## Quick Validation Commands

Examples:

```bash
# search locale references
rg -n "README\.el\.md|i18n/el/README\.md|i18n/vi/README\.md" README*.md docs/README*.md docs/SUMMARY*.md

# check changed markdown files for obvious link breaks
git status --short

# quick parity count against top-level docs baseline
base=$(find docs -maxdepth 1 -type f -name '*.md' | sed 's#^docs/##' | \
  rg -v '^(README(\..+)?\.md|SUMMARY(\..+)?\.md|commands-reference\.vi\.md|config-reference\.vi\.md|one-click-bootstrap\.vi\.md|troubleshooting\.vi\.md)$' | sort)
for loc in zh-CN ja ru fr vi el; do
  c=0
  while IFS= read -r f; do
    [ -f "docs/i18n/$loc/$f" ] || c=$((c+1))
  done <<< "$base"
  echo "$loc $c"
done
```

Use repository-preferred markdown lint/link checks when available.

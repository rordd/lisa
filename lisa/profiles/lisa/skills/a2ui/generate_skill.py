#!/usr/bin/env python3
"""Generate SKILL.md for the A2UI skill using the official Google A2UI SDK.

Usage:
    python generate_skill.py              # prints to stdout
    python generate_skill.py --write      # overwrites SKILL.md in this directory
"""

import argparse
import os
import sys

from a2ui.core.schema.constants import VERSION_0_9
from a2ui.core.schema.manager import A2uiSchemaManager
from a2ui.basic_catalog.provider import BasicCatalog
from a2ui.core.schema.common_modifiers import remove_strict_validation

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# --- Lisa-specific frontmatter (ZeroClaw skill loader) ---
FRONTMATTER = """\
---
name: a2ui
description: "A2UI v0.9 card rendering. Generate visual UI cards using Google's A2UI protocol on the WebSocket channel."
version: "5.0.0"
channels: ws
always: true
---"""

# --- Lisa-specific role description fed to SDK prompt builder ---
ROLE_DESCRIPTION = """\
# A2UI v0.9 — Agent-to-User Interface Card Rendering

When presenting structured or visual information (weather, tasks, schedules, quiz, etc.), \
you MUST include A2UI card data inside `<a2ui-json>` tags alongside your text response. \
NEVER mention a card in text without actually including the A2UI JSON block.

When the user's request does not need a card, just respond with text (no A2UI block).

When data WOULD benefit from visual display, proactively include a card. \
Cards are strongly encouraged for: weather, schedules, lists, comparisons, quizzes, choices, \
recipes, calculators, forms, travel itineraries, music playlists.

CRITICAL: When the user asks for a "카드" (card) or visual display, \
you MUST include `<a2ui-json>[...]</a2ui-json>` in your response. This is not optional. \
If you say "카드 만들었어" but don't include A2UI JSON, the user sees nothing.

## Component Diversity

Use the FULL range of components from the schema — not just Card/Column/Row/Text/Button. \
Match the component to the content:
- Checkboxes for to-do items → CheckBox
- Rating/progress → Slider
- Multiple choice selection → ChoicePicker or MultipleChoice
- Text input → TextField
- Lists of items → List
- Tabbed views → Tabs
- Images → Image
- Separators → Divider

## A2UI Action Rules

Button `action` has two types. Use the correct one:

- **`functionCall`** — runs on the CLIENT (browser). Use for:
  - Opening URLs: `{"functionCall": {"call": "openUrl", "args": {"url": "..."}, "returnType": "void"}}`
  - Formatting values: `formatString`, `formatNumber`, `formatCurrency`, `formatDate`, `pluralize`
  - Input validation: `required`, `regex`, `length`, `numeric`, `email`

- **`event`** — sends to the SERVER, which routes it back to you. Use for:
  - Choices that need your reasoning (quiz answers, preferences, follow-up questions)

CRITICAL: URL buttons MUST use `functionCall.openUrl`. The server is headless — it cannot open browsers."""

# No examples — let the LLM use the full schema freely.
# Examples cause overfitting to specific patterns (Card+Column+Row+Button)
# and prevent the LLM from using other components (CheckBox, Slider, Tabs, etc.).


def generate_skill_md() -> str:
    """Generate the full SKILL.md content."""
    # Use the official Google SDK to generate schema prompt
    schema_prompt = A2uiSchemaManager(
        VERSION_0_9,
        catalogs=[BasicCatalog.get_config(version=VERSION_0_9)],
        schema_modifiers=[remove_strict_validation],
    ).generate_system_prompt(
        role_description=ROLE_DESCRIPTION,
        include_schema=True,
        include_examples=False,  # v0.9 has no bundled examples yet
    )

    parts = [
        FRONTMATTER,
        "",
        schema_prompt,
        "",
    ]

    return "\n".join(parts)


def main():
    parser = argparse.ArgumentParser(description="Generate A2UI SKILL.md")
    parser.add_argument(
        "--write",
        action="store_true",
        help="Write output to SKILL.md (default: print to stdout)",
    )
    args = parser.parse_args()

    content = generate_skill_md()

    if args.write:
        out_path = os.path.join(SCRIPT_DIR, "SKILL.md")
        with open(out_path, "w") as f:
            f.write(content)
        print(f"Written to {out_path} ({len(content)} bytes)", file=sys.stderr)
    else:
        print(content)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Rewrites `category = "..."` lines in a learned-rules TOML file using the
`mapping:` table from category-migration.yaml.

Usage: ./migrate_categories.py [rules-file]  (default: learned-rules.local.toml)
"""
import re
import sys
from pathlib import Path

CONFIG_DIR = Path(__file__).parent
MIGRATION_FILE = CONFIG_DIR / "category-migration.yaml"
CATEGORY_LINE = re.compile(r'^(category = ")([^"]+)(")$')

# ponytail: hand-rolled instead of pyyaml (not available without a nix-shell
# detour); only handles the flat "key: value" lines under `mapping:`, which
# is all this file's mapping section ever uses. Switch to pyyaml if the
# mapping section grows nested structure.
def load_mapping(yaml_path: Path) -> dict[str, str]:
    mapping = {}
    in_mapping = False
    for raw_line in yaml_path.read_text().splitlines():
        line = raw_line.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        if line == "mapping:":
            in_mapping = True
            continue
        if in_mapping:
            if not line.startswith("  "):  # dedent ends the mapping section
                break
            key, _, value = line.strip().partition(":")
            mapping[key.strip()] = value.strip()
    return mapping


def migrate(rules_path: Path, mapping: dict[str, str]) -> tuple[str, int, set[str]]:
    updated_lines = []
    updated_count = 0
    unmapped = set()
    for line in rules_path.read_text().splitlines(keepends=False):
        match = CATEGORY_LINE.match(line)
        if match:
            old_category = match.group(2)
            new_category = mapping.get(old_category)
            if new_category is None:
                unmapped.add(old_category)
            else:
                line = f"{match.group(1)}{new_category}{match.group(3)}"
                updated_count += 1
        updated_lines.append(line)
    return "\n".join(updated_lines) + "\n", updated_count, unmapped


def demo() -> None:
    """ponytail self-check: run with `python3 migrate_categories.py --selftest`."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        (tmp / "category-migration.yaml").write_text(
            "mapping:\n  groceries: needs/groceries\n  # comment\n  cloths: wants/shopping/clothing\n\nother: 1\n"
        )
        rules = tmp / "rules.toml"
        rules.write_text(
            '[[matching_rules]]\ncategory = "groceries"\n\n'
            '[[matching_rules]]\ncategory = "unknown-legacy"\n'
        )

        mapping = load_mapping(tmp / "category-migration.yaml")
        assert mapping == {"groceries": "needs/groceries", "cloths": "wants/shopping/clothing"}, mapping

        new_text, count, unmapped = migrate(rules, mapping)
        assert count == 1, count
        assert unmapped == {"unknown-legacy"}, unmapped
        assert 'category = "needs/groceries"' in new_text
        assert 'category = "unknown-legacy"' in new_text
    print("selftest ok")


def main() -> None:
    if "--selftest" in sys.argv:
        demo()
        return

    args = [a for a in sys.argv[1:] if a != "--selftest"]
    rules_path = Path(args[0]) if args else CONFIG_DIR / "learned-rules.local.toml"

    mapping = load_mapping(MIGRATION_FILE)
    new_text, updated_count, unmapped = migrate(rules_path, mapping)

    rules_path.write_text(new_text)
    print(f"Updated {updated_count} categor{'y' if updated_count == 1 else 'ies'} in {rules_path}")
    if unmapped:
        print(f"No mapping found for (left unchanged): {', '.join(sorted(unmapped))}")


if __name__ == "__main__":
    main()

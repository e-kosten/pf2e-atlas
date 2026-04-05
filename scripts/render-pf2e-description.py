#!/usr/bin/env python3

import argparse
import html
import json
import re
import sys
from pathlib import Path


UUID_PATTERN = re.compile(r"@UUID\[([^\]]+)\](?:\{([^}]+)\})?")
CHECK_PATTERN = re.compile(r"@Check\[([^\]]+)\](?:\{([^}]+)\})?")
TEMPLATE_PATTERN = re.compile(r"@Template\[([^\]]+)\](?:\{([^}]+)\})?")
INLINE_PATTERN = re.compile(r"@([A-Z][A-Za-z]+)\[([^\]]+)\](?:\{([^}]+)\})?")
TAG_PATTERN = re.compile(r"<[^>]+>")


def extract_uuid_fallback(raw: str) -> str:
  token = raw.split(".")[-1]
  token = token.split(":")[-1]
  return token.replace("-", " ")


def replace_uuid(match: re.Match[str]) -> str:
  return match.group(2) or extract_uuid_fallback(match.group(1))


def replace_check(match: re.Match[str]) -> str:
  return match.group(2) or f"Check[{match.group(1)}]"


def replace_template(match: re.Match[str]) -> str:
  return match.group(2) or f"Template[{match.group(1)}]"


def replace_inline(match: re.Match[str]) -> str:
  return match.group(3) or f"{match.group(1)}[{match.group(2)}]"


def render_description(value: str, strip_foundry: bool) -> str:
  rendered = re.sub(r"<hr ?/?>", "\n---\n", value, flags=re.IGNORECASE)
  rendered = re.sub(r"</p>\s*", "\n\n", rendered, flags=re.IGNORECASE)
  rendered = re.sub(r"<br ?/?>", "\n", rendered, flags=re.IGNORECASE)
  rendered = TAG_PATTERN.sub("", rendered)
  rendered = html.unescape(rendered)

  if strip_foundry:
    rendered = UUID_PATTERN.sub(replace_uuid, rendered)
    rendered = CHECK_PATTERN.sub(replace_check, rendered)
    rendered = TEMPLATE_PATTERN.sub(replace_template, rendered)
    rendered = INLINE_PATTERN.sub(replace_inline, rendered)

  rendered = re.sub(r"\n{3,}", "\n\n", rendered)
  return rendered.strip()


def main() -> int:
  parser = argparse.ArgumentParser(
    description="Render a PF2E Foundry JSON description as readable plain text.",
  )
  parser.add_argument("path", help="Path to a PF2E JSON record file")
  parser.add_argument(
    "--keep-foundry",
    action="store_true",
    help="Keep @UUID/@Check/@Template markup instead of stripping it",
  )
  args = parser.parse_args()

  path = Path(args.path)
  with path.open("r", encoding="utf-8") as handle:
    payload = json.load(handle)

  try:
    value = payload["system"]["description"]["value"]
  except KeyError as exc:
    print(f"Missing description value in {path}: {exc}", file=sys.stderr)
    return 1

  print(render_description(value, strip_foundry=not args.keep_foundry))
  return 0


if __name__ == "__main__":
  raise SystemExit(main())

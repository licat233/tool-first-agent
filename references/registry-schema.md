# Registry Schema

`registry/tools.yaml` is the local source of truth for candidate tools.

Top-level keys are category names. Each category contains `tools`.

```yaml
document:
  description: "Document extraction and conversion"
  tools:
    pandoc:
      priority: 20
      detect_names: [pandoc]
      version_args: ["--version"]
      handles:
        - "Convert docx/markdown/html/epub"
      commands:
        extract_docx_text: "pandoc {input} -t plain"
      fallbacks: [markitdown, libreoffice, textutil]
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `priority` | int | Lower number = try earlier |
| `detect_names` | list[string] | Executable names to check |
| `known_paths` | list[string] | Exact absolute paths to check |
| `app_bundle_paths` | list[string] | macOS app bundle executable paths |
| `version_args` | list[string] | Lightweight version command arguments |
| `handles` | list[string] | Short capability notes |
| `commands` | map[string, string] | Reusable command templates with `{input}`, `{output}`, `{url}` placeholders |
| `fallbacks` | list[string] | Other tool keys to consider |

## Categories

10 categories: `document`, `pdf`, `image`, `media`, `data`, `search`, `archive`, `dev`, `web`, `ai`.

Keep entries concise. This registry is for task routing, not man pages.

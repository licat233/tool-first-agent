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

- `priority`: lower number means try earlier.
- `detect_names`: executable names to check.
- `known_paths`: exact absolute paths to check.
- `app_bundle_paths`: exact macOS app bundle executable paths.
- `version_args`: lightweight version command arguments.
- `handles`: short capability notes.
- `commands`: reusable command templates.
- `fallbacks`: other tool keys to consider.

Keep entries concise. This registry is for task routing, not man pages.

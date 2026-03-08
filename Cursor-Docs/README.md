# Cursor-Docs

This folder holds **implementation and historical notes** for developers and AI assistants. They are not the primary user documentation.

**Primary docs (use these for setup and usage):**

- [README.md](../README.md) — Overview, installation, CLI usage, manifest format
- [QUICKSTART.md](../QUICKSTART.md) — Short path to run CLI or GUI
- [SETUP.md](../SETUP.md) — One-time setup and verification
- [PROJECT_STRUCTURE.md](../PROJECT_STRUCTURE.md) — Workspace layout and doc map
- [DEVELOPMENT.md](../DEVELOPMENT.md) — Development workflow
- [CONTRIBUTING.md](../CONTRIBUTING.md) — Contribution guidelines
- [crtool-cli/README.md](../crtool-cli/README.md), [crtool-gui/README.md](../crtool-gui/README.md) — Per-tool docs

**Contents of this folder:**

| File | Description |
|------|-------------|
| [VALIDATION_FEATURE.md](VALIDATION_FEATURE.md) | Implementation notes for `--validate` (crJSON schema) |
| [GUI_IMPLEMENTATION.md](GUI_IMPLEMENTATION.md) | GUI architecture and library API |
| [GUI_SETUP_COMPLETE.md](GUI_SETUP_COMPLETE.md) | Historical GUI setup summary |
| [INGREDIENT_METADATA_ISSUE.md](INGREDIENT_METADATA_ISSUE.md) | Ingredient metadata support — problem and resolution |
| [INGREDIENT_METADATA_IMPLEMENTATION.md](INGREDIENT_METADATA_IMPLEMENTATION.md) | Ingredient metadata implementation details |
| [INGREDIENT_IMPLEMENTATION.md](INGREDIENT_IMPLEMENTATION.md) | File-based ingredients (`ingredients_from_files`) design |
| [INGREDIENT_LABEL_SUPPORT.md](INGREDIENT_LABEL_SUPPORT.md) | Ingredient `label` field for action references |

User-facing behavior for validation, ingredients, and GUI is documented in the root README and the linked primary docs above.

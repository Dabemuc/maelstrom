# Architecture Goal

This project is organized as a layered Rust workspace with clear boundaries between
UI apps, service orchestration, and core domain crates. The goal is to keep UI apps
lightweight and move cross-cutting orchestration into a single services crate.

## Layers

1. Apps (UI / CLI)

- Desktop app (Iced) and CLI are thin shells.
- They depend on the `services` crate (and UI/CLI framework crates only).
- Apps do not depend directly on low-level domain crates.

2. Services

- A single entry point for application use-cases.
- Orchestrates IO, preview generation, catalog access, and edit graph application.
- Owns background task orchestration (thread pool or async runtime integration).
- Maps low-level errors into app-friendly errors.

3. Domain Crates

- `io`: catalog persistence, filesystem scanning, metadata, import.
- `graph`: node graph for image operations.
- `ops`: operator implementations (exposure, white balance, etc.).
- `image`: `LinearImage` and conversions.
- `previews`: preview generation and caching.
- `core`: color, hashing, and shared utilities.

## Dependencies

- Apps -> services
- Services -> domain crates
- Domain crates -> core (as needed)

This keeps the dependency graph shallow at the app level and makes it easier to
share workflows between desktop and CLI while keeping IO and processing logic
consistent.

## Background Work

All long-running work (filesystem scans, preview generation, import processing,
catalog queries) should be initiated through services APIs. Apps should only
dispatch requests and react to results.

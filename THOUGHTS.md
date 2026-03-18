# Architecture rules

- UI Layer (components) only renders state
- Business logic should be in seperate layer so can be reused by different apps (desktop/cli).
- Crates should be more encapsulated (Maybe service structs that get composed in a setup?)

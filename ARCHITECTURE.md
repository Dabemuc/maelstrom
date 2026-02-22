# Note

This was ai genereted to get a starting point for the project.

Below is a concrete, production-oriented crate layout for a Rust-based Lightroom-style application. The design assumes:

- High-performance modular monolith core
- Stable plugin API via C ABI or WASM
- GPU acceleration
- Non-destructive parametric processing graph
- Background rendering + catalog system

---

# Top-Level Workspace Layout

```
lightroom-rs/
│
├── apps/
│   ├── desktop/                # GUI app
│   └── cli/                    # Headless renderer / batch export
│
├── crates/
│   ├── core/
│   ├── image/
│   ├── color/
│   ├── graph/
│   ├── gpu/
│   ├── cache/
│   ├── catalog/
│   ├── jobs/
│   ├── raw/
│   ├── ops/
│   ├── preview/
│   ├── export/
│   ├── plugin_api/
│   ├── plugin_host/
│   └── ui/
│
└── plugins/
    ├── export_jpeg/
    ├── export_avif/
    ├── ai_denoise/
    └── lens_corrections/
```

---

# Core Architectural Layers

## 1. `core` (Foundation Layer)

**Responsibility:** Shared primitives and engine-wide traits.

Contains:

- Strong ID types (ImageId, NodeId, AssetId)
- Versioned parameter structs
- Engine error types
- Global configuration
- Trait definitions for:

  - ProcessingOperator
  - RenderBackend
  - Plugin
  - Exporter

This crate has **no GPU or UI dependencies**.

---

## 2. `image` (Image Buffer & Pixel Types)

**Responsibility:** Core image representations.

Types:

- RawImage (Bayer/X-Trans data)
- LinearImage (f32 linear working space)
- RgbImage<T>
- TiledImage
- ImagePyramid

Key decisions:

- Use f32 internally for linear processing
- Tile-based memory layout
- Optional SIMD acceleration

This crate must be highly optimized and dependency-light.

---

## 3. `color` (Color Management)

**Responsibility:** Color science pipeline.

Contains:

- ICC profile parsing
- Camera profiles
- Working space transforms
- Output transforms (sRGB, Display P3, etc.)
- Tone curves
- LUT support

This is a core engine piece and should not be pluggable.

---

## 4. `graph` (Non-Destructive Processing DAG)

This is the heart of the system.

**Responsibility:**

- Parametric processing graph (DAG)
- Immutable edit state
- Dependency tracking
- Partial recomputation
- Hash-based node caching

Example conceptual model:

```
RAW Decode
   ↓
Demosaic
   ↓
White Balance
   ↓
Exposure
   ↓
Tone Curve
   ↓
Local Mask Node
   ↓
Output Transform
```

Key components:

- Node trait (pure function over image + params)
- Graph struct
- Topological scheduler
- Change propagation system

This crate must be highly deterministic and thread-safe.

---

## 5. `raw` (RAW Decoding)

**Responsibility:**

- Camera file parsing (CR3, NEF, ARW)
- Metadata extraction
- Demosaicing algorithms
- Camera-specific color matrices

Architecture:

- Core trait: RawDecoder
- Each camera format as module
- Possibly split into subcrates for licensing separation

This module feeds into `image::RawImage`.

---

## 6. `ops` (Image Processing Operators)

**Responsibility:** Parametric operations.

Examples:

- Exposure
- Contrast
- Highlights/Shadows
- Tone curve
- HSL
- Clarity
- Sharpening
- Noise reduction
- Lens correction
- Mask blending

Each operator:

- Stateless
- Pure function
- Accepts params struct
- Returns new image (or tile)

Operators register into the graph system.

These are compiled into the engine (not runtime plugins) for performance.

---

## 7. `gpu` (GPU Abstraction Layer)

**Responsibility:**

- Compute abstraction (Metal/Vulkan/DirectX/wgpu)
- Shader management
- GPU memory pools
- Tile dispatch

Expose:

```
trait GpuKernel {
    fn dispatch(&self, input: &GpuImage, params: &Params) -> GpuImage;
}
```

CPU fallback must exist.

GPU must integrate tightly with `graph` for scheduling.

---

## 8. `cache` (Render Cache)

**Responsibility:**

- Tile cache
- Pyramid cache
- Disk cache
- Hash-based node caching
- Memory pressure handling

Lightroom-style preview requires:

- Multi-resolution pyramid
- Cached intermediate nodes
- Tile-based invalidation

This crate is critical for performance.

---

## 9. `preview` (Viewport Rendering)

**Responsibility:**

- Tile rendering for zoomed view
- Region-of-interest recompute
- Progressive refinement
- GPU-backed display surfaces

Should integrate with UI but not depend on it.

---

## 10. `jobs` (Background Processing)

**Responsibility:**

- Thread pool
- Render queue
- Export queue
- Thumbnail generation
- Priority scheduling

Lightroom-like systems require:

- Interactive tasks (high priority)
- Background previews
- Batch export tasks

This should support cancellation and dependency awareness.

---

## 11. `catalog` (Asset Database)

**Responsibility:**

- Image metadata
- EXIF/IPTC
- Ratings
- Collections
- Keywords
- Edit parameter storage
- Smart collections (query engine)

Likely backed by SQLite.

Architecture:

- Repository pattern
- Strongly typed query layer
- Schema migrations

Catalog must store graph parameter state, not rendered images.

---

## 12. `export`

**Responsibility:**

- Format abstraction
- Color conversion
- Resizing
- Sharpen-for-output
- Metadata writing

Trait example:

```
trait Exporter {
    fn export(&self, image: &LinearImage, settings: ExportSettings) -> Result<()>;
}
```

Exporters can be plugins.

---

## 13. Plugin System

### `plugin_api`

Stable ABI crate.

Contains:

- C-compatible types
- Versioned structs
- Capability negotiation
- Host function table

Example capability categories:

- Exporter
- Metadata provider
- AI operator
- UI panel
- Cloud integration

Should avoid exposing internal engine types directly.

---

### `plugin_host`

Responsible for:

- Dynamic loading
- WASM runtime (optional)
- Sandboxing
- Plugin lifecycle
- API version negotiation

Plugins must not touch internal memory directly.

---

## 14. `ui` (Frontend Layer)

Choice of toolkit:

- egui
- iced
- wgpu-based custom
- or native bindings

UI talks only to:

- graph
- catalog
- preview
- jobs

Never directly to low-level image internals.

---

# Data Flow Overview

```
File Import
   ↓
raw::decode()
   ↓
image::RawImage
   ↓
graph execution
   ↓
cache
   ↓
preview tiles
   ↓
UI
```

Export path:

```
graph full render
   ↓
color transform
   ↓
exporter
```

---

# Performance-Critical Rules

1. Graph nodes must be pure.
2. Image buffers must avoid reallocations.
3. Tile-based processing everywhere.
4. Hash parameters for cache invalidation.
5. Avoid dynamic dispatch in inner loops.
6. GPU path must share graph structure.

---

# What Is NOT Pluggable

To maintain performance, these should stay in-core:

- Processing graph engine
- Image buffer implementation
- GPU abstraction
- Core color management
- Render scheduler

Plugins should extend, not redefine, the engine.

---

# Optional Advanced Ideas

- WASM plugin sandbox
- Node-based visual graph editor
- Remote render worker
- Distributed export cluster
- Live collaboration layer
- Edit history as CRDT

---

# Architectural Philosophy Summary

- A high-performance core engine (modular monolith)
- Strict internal layering
- Stable, narrow plugin boundary
- Deterministic processing graph
- Tile-based renderer

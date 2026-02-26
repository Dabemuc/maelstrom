# Next steps
## Catalog
  - [x] Turso sqlite file that contains:
    - [x] info about imported directories
    - [ ] metadate of imported files
      - [ ] Nodegraph (if exists)
      - [x] preview file location
    - [ ] settings
  - [x] Maelstrom needs an active catalog -> On startup if no catalog found in default location make the user create one or import existing one
    - [x] no_catalog_view 
    - [x] default catalog location and loading on startup
  - [ ] catalog dropdown in top left to access load/create/any future functionality
- [x] Working directory
- [ ] Rework workspace state to be hierarchical
  - [ ] -> Keep previews in state
  - [ ] -> integrate image counts into workspace state
## Sidebars
- [x] Navigator
  - [x] shows all imported directories from catalog 
  - [x] lets user set Working directory by clicking
  - [ ] Opption to remove directory
- [ ] Scalable sidebars -> Store in settings table
## Library view
  - [x] Generate and cache or load previews for current working directory -> good enough for now
  - [x] Show them in library view 
  - [ ] Order by selector (name, create date, ...)
  - [ ] Build Nodes to support preview generation graph
    - [ ] Resolution and Compress maybe?
    - [x] Store previews as jpeg
    - [x] TODO: Rework preview generation and loading. Should be like this: 
      - Load previews for folder asynchronously all at once -> Put them into state independently of each other
      - In background start cache refresh to check if cache matches images in filesystem
    - [ ] Cache Refresh Progress bar
    - [ ] Enable user to handle missing image
  - [ ] Image selection
  - [ ] Image metadata in right sidebar
  - [ ] User can switch to Develop view by double clicking foto
  ## Develop view
  - [ ] Interactions modify node graph -> Rerendering of developed foto and its preview
  - [ ] ...


# For the future
- Replace linear pipeline with DAG
  - caching, partial recomputation, parallell computation
  - But also required for features: Branching, Intermediate results caching, mask subgraphs, local adjustments, selective rerendering
- RAW loading
- Memory-mapped Previews? RGBA raw bytes or BC7 compressed textures.
- Tiling
- Cpu multithreaded backend
- GPU backend
- Serialize edit stack -> enables presets
- UI
  - Historgram node
  - Parameter system for nodes: Nodes define which types of params they need (float{min,max,default}, Bool, Curve) and UI renders input accordingly
  - Edit History and Undo/Redo
  - Tone Curve Node
  - Preview scaling on slider drags (compute low res preview first to keep adjusting smooth and snappy)
  - Masks
  
# Keep in mind
- Nodes should stay pure and stateless

# Next steps
- [ ] Catalog 
  - [x] Turso sqlite file that contains:
    - [x] info about imported directories
    - [ ] metadate of imported files
      - [ ] Nodegraph (if exists)
      - [ ] preview file location
    - [ ] settings
  - [x] Maelstrom needs an active catalog -> On startup if no catalog found in default location make the user create one or import existing one
    - [x] no_catalog_view 
    - [x] default catalog location and loading on startup
  - [ ] catalog dropdown in top left to access load/create/any future functionality
- [x] Navigator shows all imported directories from catalog and lets user set Working directory by clicking
- [x] Working directory
- [ ] Library view
  - [x] Generate and cache or load previews for current working directory -> good enough for now
  - [ ] Show them in library view 
  - [ ] Build Nodes to support preview generation graph
    - [ ] Resolution and Compress maybe?
    - [ ] Store previews as jpeg
  - [ ] User can switch to Develop view by double clicking foto
- [ ] Develop view
  - [ ] Interactions modify node graph -> Rerendering of developed foto and its preview
  - [ ] ...


# For the future
- Replace linear pipeline with DAG
  - caching, partial recomputation, parallell computation
  - But also required for features: Branching, Intermediate results caching, mask subgraphs, local adjustments, selective rerendering
- RAW loading
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

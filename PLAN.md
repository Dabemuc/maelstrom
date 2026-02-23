# Next steps
- [ ] Catalog 
  - [x] Turso sqlite file that contains:
    - [x] info about imported directories
    - [ ] metadate of imported files
      - [ ] Nodegraph (if exists)
      - [ ] preview file location
    - [ ] settings
  - [ ] Maelstrom needs an active catalog -> On startup if no catalog found in default location make the user create one or import existing one
    - [x] no_catalog_view 
    - [ ] actually handle button clicks by openeing finder
- [ ] Navigator shows all imported directories from catalog and lets user set Working directory by clicking
- [ ] Working directory
- [ ] Library view
  - [ ] Show only fotos from current working directory in library view
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

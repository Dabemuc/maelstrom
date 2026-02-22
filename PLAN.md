# Next steps
- [ ] Proper color management
- [ ] 
- [ ] 


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
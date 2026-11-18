- **`El::backdrop_blur` now blurs on the web.** A WebGPU canvas is configured with `COPY_SRC`,
  so the renderer can copy what is drawn under a blurred element. Before, the web backend asked
  for it only when wgpu listed it, which wgpu never does for a browser canvas, so on the web a
  backdrop blur drew only its fill (the `backdrop_blur` entry said the web asked for it). WebGL 2
  pages are unchanged: they still draw only the fill.

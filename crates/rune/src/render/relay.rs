//! A frame for a target that can't be copied from (a WebGL 2 canvas: wgpu offers its surface only
//! as a render target). A frame with a backdrop effect draws into a texture of the renderer's own,
//! which can be copied, and one more pass puts that texture on the target.
use super::Renderer;

pub(super) struct Relay {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    blitter: wgpu::util::TextureBlitter,
    format: wgpu::TextureFormat,
}

impl Relay {
    fn texture(device: &wgpu::Device, format: wgpu::TextureFormat, (width, height): (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("relayed frame"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        (texture, view)
    }
}

impl Renderer {
    /// [`Self::draw`] into a `view` of the renderer's format that may lack `COPY_SRC`. A frame
    /// with a backdrop effect draws into a texture the renderer keeps, which can be copied, and
    /// is then drawn onto `view`, so its effects draw what is under them where `draw` draws only
    /// their fill. Every other frame is `draw`. True when the frame took the relay.
    #[doc(hidden)]
    pub fn draw_relayed(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, clear: [f32; 4]) -> bool {
        let target = view.texture();
        if !self.backdrop_used || target.usage().contains(wgpu::TextureUsages::COPY_SRC) {
            self.draw(encoder, view, clear);
            return false;
        }
        let (format, size) = (self.format, (target.width(), target.height()));
        let relay = match self.relay.take() {
            Some(mut r) if r.format == format => {
                if (r.texture.width(), r.texture.height()) != size { (r.texture, r.view) = Relay::texture(device, format, size); }
                r
            }
            _ => {
                let (texture, view) = Relay::texture(device, format, size);
                Relay { texture, view, blitter: wgpu::util::TextureBlitter::new(device, format), format }
            }
        };
        self.draw(encoder, &relay.view, clear);
        relay.blitter.copy(device, encoder, &relay.view, view);
        self.relay = Some(relay);
        true
    }
}

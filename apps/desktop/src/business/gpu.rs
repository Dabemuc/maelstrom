use std::sync::Arc;

use iced::Rectangle;
use iced_wgpu::graphics::Viewport;
use iced_wgpu::primitive::{self, Pipeline};
use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use maelstrom_image::linear_image::LinearImage;

#[derive(Debug, Clone)]
pub struct LinearImagePrimitive {
    pub image: Arc<LinearImage>,
}

impl LinearImagePrimitive {
    pub fn new(image: Arc<LinearImage>) -> Self {
        Self { image }
    }
}

impl primitive::Primitive for LinearImagePrimitive {
    type Pipeline = LinearImagePipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        pipeline.prepare(device, queue, &self.image);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        pipeline.draw(render_pass);
        true
    }
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct GammaSettings {
    apply_srgb: u32,
    _padding: [u32; 3],
}

#[derive(Debug)]
struct CachedTexture {
    image: Arc<LinearImage>,
    _texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct LinearImagePipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    gamma_buffer: wgpu::Buffer,
    cached_texture: Option<CachedTexture>,
}

impl LinearImagePipeline {
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, image: &Arc<LinearImage>) {
        if let Some(cache) = &self.cached_texture {
            if Arc::ptr_eq(&cache.image, image) {
                return;
            }
        }

        let size = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("linear-image"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let bytes_per_row = image.stride * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row = (bytes_per_row + align - 1) / align * align;

        if padded_bytes_per_row == bytes_per_row {
            queue.write_texture(
                texture.as_image_copy(),
                bytemuck::cast_slice(&image.data),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: None,
                },
                size,
            );
        } else {
            let height = image.height as usize;
            let src = bytemuck::cast_slice(&image.data);
            let mut padded = vec![0u8; padded_bytes_per_row * height];

            for row in 0..height {
                let src_offset = row * bytes_per_row;
                let dst_offset = row * padded_bytes_per_row;
                padded[dst_offset..dst_offset + bytes_per_row]
                    .copy_from_slice(&src[src_offset..src_offset + bytes_per_row]);
            }

            queue.write_texture(
                texture.as_image_copy(),
                &padded,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
                size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("linear-image.bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.gamma_buffer.as_entire_binding(),
                },
            ],
        });

        self.cached_texture = Some(CachedTexture {
            image: Arc::clone(image),
            _texture: texture,
            bind_group,
        });
    }

    fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        let Some(cache) = &self.cached_texture else {
            return;
        };

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &cache.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

impl Pipeline for LinearImagePipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        #[repr(C)]
        #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct Vertex {
            position: [f32; 2],
            uv: [f32; 2],
        }

        let vertices = [
            Vertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                uv: [0.0, 0.0],
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("linear-image.vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("linear-image.indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let gamma_settings = GammaSettings {
            apply_srgb: if format.is_srgb() { 0 } else { 1 },
            _padding: [0; 3],
        };
        let gamma_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("linear-image.gamma"),
            contents: bytemuck::bytes_of(&gamma_settings),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("linear-image.bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/linear_image.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("linear-image.pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("linear-image.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            gamma_buffer,
            cached_texture: None,
        }
    }
}

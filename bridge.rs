// SEAM 2, solved: the ESP bridge between the game crate and the render crate.
//
// This payload is added as a `visibility = "pub"` support module to
// `crates/horizon-client-render/src/gfx/native_render.rs`, i.e. it lives in
// `horizon_client_render::gfx::native_render::meteor_bridge`.
//
// That direction is the only one Cargo allows: `horizon-client` depends on
// `horizon-client-render`, never the other way. So the render crate owns the
// state and the *game* crate writes into it:
//
//   game side (tick.rs, `crate::game::Game<'_>`):
//     horizon_client_render::gfx::native_render::meteor_bridge::set_boxes(boxes)
//   render side (world_drawn.rs, same module):
//     meteor_bridge::boxes() / meteor_bridge::pipeline(..)
//
// The two run on different threads (tick thread vs render thread), so the
// boxes are behind a `Mutex`. A poisoned lock is treated as "nothing to draw"
// rather than a panic: a mod must never take the renderer down.

use std::sync::{Mutex, OnceLock};

/// One ESP target box, already camera-independent: a world position and size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Box3 {
    /// The player's feet, in world coordinates.
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Full width and depth of the box, in blocks.
    pub width: f32,
    /// Full height, in blocks (1.8 for a standing player).
    pub height: f32,
    /// Linear RGBA.
    pub color: [f32; 4],
}

/// The boxes the game side last published for this frame set.
static BOXES: Mutex<Vec<Box3>> = Mutex::new(Vec::new());

/// Publishes the boxes the renderer should draw. Called on the tick thread.
pub fn set_boxes(boxes: Vec<Box3>) {
    match BOXES.lock() {
        Ok(mut current) => {
            *current = boxes;
        }
        Err(poisoned) => {
            // A previous holder panicked. Recover rather than cascade.
            *poisoned.into_inner() = boxes;
        }
    }
}

/// The boxes to draw now. Called on the render thread.
pub fn boxes() -> Vec<Box3> {
    match BOXES.lock() {
        Ok(current) => current.clone(),
        Err(_) => Vec::new(),
    }
}

/// Drops every box: the world changed, so they mean nothing.
pub fn clear() {
    set_boxes(Vec::new());
}

// ------------------------------------------------------------------ pipeline --

/// The vertex layout the ESP shader takes: a position already transformed to
/// clip space on the CPU, plus a colour. Written as raw floats so the buffer
/// can be built without a vertex type shared with the game.
pub const VERTEX_STRIDE: u64 = 8 * 4;

/// The pipeline and its bind-free layout, built once per device.
pub struct EspResources {
    pub pipeline: wgpu::RenderPipeline,
}

/// One cache per colour format: the game can switch targets (HDR, sRGB).
static PIPELINE: OnceLock<Mutex<Option<(wgpu::TextureFormat, &'static EspResources)>>> =
    OnceLock::new();

fn cache() -> &'static Mutex<Option<(wgpu::TextureFormat, &'static EspResources)>> {
    PIPELINE.get_or_init(|| Mutex::new(None))
}

const SHADER: &str = r#"
// Clip-space positions arrive already transformed; this only forwards them.
struct In {
    @location(0) clip: vec4<f32>,
    @location(1) color: vec4<f32>,
};
struct Out {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};
@vertex
fn vs(input: In) -> Out {
    var out: Out;
    out.pos = input.clip;
    out.color = input.color;
    return out;
}
@fragment
fn fs(input: Out) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

/// The pipeline for `format`, built on first use and kept for the process.
/// Returns `None` rather than panicking if the device refuses the pipeline.
pub fn pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Option<&'static EspResources> {
    let guard = cache();
    let mut slot = match guard.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some((cached_format, resources)) = *slot {
        if cached_format == format {
            return Some(resources);
        }
    }

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("meteor esp shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("meteor esp layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("meteor esp pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: VERTEX_STRIDE,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: 1,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveTopology::LineList.into(),
        // Depth-tested against the world's own depth: reversed, 0 is the sky,
        // so the comparison is Greater and a nearer fragment wins.
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let leaked: &'static EspResources = Box::leak(Box::new(EspResources { pipeline }));
    *slot = Some((format, leaked));
    Some(leaked)
}

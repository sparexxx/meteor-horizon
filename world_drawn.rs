// Genuine 3D Player ESP, drawn over the finished world.
//
// Compiled inside `horizon_client_render::gfx::native_render::native_world_drawn`,
// so `meteor_bridge` (the support module added to this same file by bridge.rs)
// is a sibling module and resolves directly.
//
// No game state is touched here: the tick thread published the boxes through
// the bridge, and this only transforms and draws them. The depth test uses the
// world's own depth buffer, so boxes are hidden behind terrain instead of
// floating on top of it.
fn meteor_world_drawn(world: &mut crate::gfx::native_render::WorldDrawn<'_>) {
    let boxes = meteor_bridge::boxes();
    if boxes.is_empty() {
        return;
    }

    // Vertices are built in bytes so no vertex type or extra crate is needed:
    // eight f32s per vertex (clip x,y,z,w then r,g,b,a), two vertices a line.
    let mut bytes: Vec<u8> = Vec::with_capacity(boxes.len() * 24 * 32);
    let mut vertex_count: u32 = 0;

    for b in &boxes {
        // World position to camera-relative, which is what view_proj expects.
        let cx = (b.x - world.camera.x) as f32;
        let cy = (b.y - world.camera.y) as f32;
        let cz = (b.z - world.camera.z) as f32;
        let hw = b.width * 0.5;
        let y0 = cy;
        let y1 = cy + b.height;
        let corners = [
            [cx - hw, y0, cz - hw],
            [cx + hw, y0, cz - hw],
            [cx + hw, y0, cz + hw],
            [cx - hw, y0, cz + hw],
            [cx - hw, y1, cz - hw],
            [cx + hw, y1, cz - hw],
            [cx + hw, y1, cz + hw],
            [cx - hw, y1, cz + hw],
        ];
        const EDGES: [(usize, usize); 12] = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        for (a, b2) in EDGES {
            for corner in [corners[a], corners[b2]] {
                let clip = world.view_proj * glam::Vec4::new(corner[0], corner[1], corner[2], 1.0);
                for v in [clip.x, clip.y, clip.z, clip.w] {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
                for c in b.color {
                    bytes.extend_from_slice(&c.to_le_bytes());
                }
                vertex_count += 1;
            }
        }
    }

    let Some(resources) = meteor_bridge::pipeline(world.device, world.color_format) else {
        return;
    };

    let buffer = world.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("meteor esp vertices"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    world.queue.write_buffer(&buffer, 0, &bytes);

    let mut pass = world.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("meteor esp"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: world.color_view,
            resolve_target: None,
            ops: wgpu::Operations {
                // Load, not Clear: the world is already drawn underneath.
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: world.depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_pipeline(&resources.pipeline);
    pass.set_vertex_buffer(0, buffer.slice(..));
    pass.draw(0..vertex_count, 0..1);
}

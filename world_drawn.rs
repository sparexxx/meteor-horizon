// Genuine 3D Player ESP, drawn over the finished world.
//
// This helper is compiled inside `horizon_client_render::gfx::native_render::
// native_world_drawn`, i.e. in the *render* crate. That crate cannot see
// `crate::game::official_meteor_horizon` (the game crate depends on the render
// crate, never the other way), so the target list arrives through a small
// bridge module that the bundle adds to the render crate — SEAM 2b in the
// README explains exactly what to add. Everything drawn here is real geometry:
// twelve camera-relative line segments per box, transformed by the frame's own
// `view_proj` and depth-tested against `world.depth`.
fn meteor_world_drawn(world: &mut crate::gfx::native_render::WorldDrawn<'_>) {
    // The bridge: the render crate's copy of the ESP targets.
    let Some(boxes) = crate::gfx::meteor_bridge::boxes() else {
        return;
    };
    if boxes.is_empty() {
        return;
    }

    // Camera-relative corners of each box, as clip space.
    let mut verts: Vec<[f32; 3]> = Vec::with_capacity(boxes.len() * 24);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(boxes.len() * 24);
    for b in &boxes {
        // World position to camera-relative.
        let x = (b.x - world.camera.x) as f32;
        let y = (b.y - world.camera.y) as f32;
        let z = (b.z - world.camera.z) as f32;
        let (w, h) = (b.width, b.height);
        let min = [x - w * 0.5, y, z - w * 0.5];
        let max = [x + w * 0.5, y + h, z + w * 0.5];
        let c = b.color;
        push_box(&mut verts, &mut colors, min, max, c);
    }

    // Every pair of vertices is one line.
    let count = verts.len() as u32;

    // Positions are transformed on the CPU with the frame's own matrix, so the
    // shader stays trivial and needs no uniform buffer of its own.
    let mut gpu: Vec<([f32; 4], [f32; 4])> = Vec::with_capacity(verts.len());
    for (i, p) in verts.iter().enumerate() {
        let v = world.view_proj * glam::Vec4::new(p[0], p[1], p[2], 1.0);
        gpu.push(([v.x, v.y, v.z, v.w], colors[i]));
    }

    if let Some(resources) = crate::gfx::meteor_bridge::pipeline(
        world.device,
        world.color_format,
    ) {
        // The buffer is rebuilt per frame: the ESP target set is tiny (a few
        // dozen boxes at most) and this keeps the renderer free of retained
        // state that could outlive the world it was built for.
        use wgpu::util::DeviceExt;
        let buffer = world.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("meteor esp lines"),
            contents: bytemuck::cast_slice(&gpu),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut pass = world.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("meteor esp"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: world.color_view,
                resolve_target: None,
                ops: wgpu::Operations {
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
        pass.draw(0..count, 0..1);
    }
}

/// Twelve edges of an axis-aligned box, as camera-relative endpoints.
fn push_box(
    verts: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    min: [f32; 3],
    max: [f32; 3],
    c: [f32; 4],
) {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;
    let corners = [
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y0, z1],
        [x0, y0, z1],
        [x0, y1, z0],
        [x1, y1, z0],
        [x1, y1, z1],
        [x0, y1, z1],
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
    for (a, b) in EDGES {
        verts.push(corners[a]);
        verts.push(corners[b]);
        colors.push(c);
        colors.push(c);
    }
}

use three_d::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let window = Window::new(WindowSettings {
        title: "DAE!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl().unwrap();

    // Renderer
    let target = vec3(0.0, 2.0, 0.0);
    let scene_radius = 6.0;
    let mut camera = Camera::new_perspective(
        &context,
        window.viewport().unwrap(),
        target + scene_radius * vec3(0.6, 0.3, 1.0).normalize(),
        target,
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    )
    .unwrap();

    Loader::load(
        &[
            "./examples/assets/sphere.dae",
        ],
        move |mut loaded| {
            let (mut meshes, _materials) = loaded.dae("./examples/assets/sphere.dae").unwrap();
            let cpu_mesh = meshes.remove(0);
            let mut model = Mesh::new(
                &context,
                &cpu_mesh,
            )
            .unwrap();
            model.transformation = Mat4::from_translation(vec3(0.0, 2.0, 0.0));

            let ambient_light = AmbientLight {
                intensity: 0.7,
                color: vec3(1.0, 1.0, 1.0),
            };
            let directional_light0 =
                DirectionalLight::new(&context, 1.0, &vec3(1.0, 1.0, 1.0), &vec3(-1.0, -1.0, -1.0))
                    .unwrap();
            let directional_light1 =
                DirectionalLight::new(&context, 1.0, &vec3(1.0, 1.0, 1.0), &vec3(1.0, 1.0, 1.0))
                    .unwrap();

            // main loop
            window
                .render_loop(move |frame_input| {
                    let mut redraw = frame_input.first_frame;
                    redraw |= camera.set_viewport(frame_input.viewport).unwrap();

                    for event in frame_input.events.iter() {
                        match event {
                            Event::MouseMotion { delta, button, .. } => {
                                if *button == Some(MouseButton::Left) {
                                    camera
                                        .rotate_around_with_fixed_up(
                                            &target,
                                            0.1 * delta.0 as f32,
                                            0.1 * delta.1 as f32,
                                        )
                                        .unwrap();
                                    redraw = true;
                                }
                            }
                            Event::MouseWheel { delta, .. } => {
                                camera
                                    .zoom_towards(&target, 0.1 * delta.1 as f32, 1.0, 100.0)
                                    .unwrap();
                                redraw = true;
                            }
                            _ => {}
                        }
                    }

                    if redraw {
                        Screen::write(
                            &context,
                            ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0),
                            || {
                                model.render_with_lighting(
                                    RenderStates::default(),
                                    &camera,
                                    Some(&ambient_light),
                                    &[&directional_light0, &directional_light1],
                                    &[],
                                    &[],
                                )?;

                                Ok(())
                            },
                        )
                        .unwrap();
                    }

                    if args.len() > 1 {
                        // To automatically generate screenshots of the examples, can safely be ignored.
                        FrameOutput {
                            screenshot: Some(args[1].clone().into()),
                            exit: true,
                            ..Default::default()
                        }
                    } else {
                        FrameOutput {
                            swap_buffers: redraw,
                            wait_next_event: true,
                            ..Default::default()
                        }
                    }
                })
                .unwrap();
        },
    );
}

fn vertex_transformations(cpu_mesh: &CPUMesh) -> Vec<Mat4> {
    let mut iter = cpu_mesh.positions.iter();
    let mut vertex_transformations = Vec::new();
    while let Some(v) = iter.next() {
        vertex_transformations.push(Mat4::from_translation(vec3(
            *v,
            *iter.next().unwrap(),
            *iter.next().unwrap(),
        )));
    }
    vertex_transformations
}

fn edge_transformations(cpu_mesh: &CPUMesh) -> Vec<Mat4> {
    let mut edge_transformations = std::collections::HashMap::new();
    let indices = cpu_mesh.indices.as_ref().unwrap().into_u32();
    for f in 0..indices.len() / 3 {
        let mut fun = |i1, i2| {
            let p1 = vec3(
                cpu_mesh.positions[i1 * 3],
                cpu_mesh.positions[i1 * 3 + 1],
                cpu_mesh.positions[i1 * 3 + 2],
            );
            let p2 = vec3(
                cpu_mesh.positions[i2 * 3],
                cpu_mesh.positions[i2 * 3 + 1],
                cpu_mesh.positions[i2 * 3 + 2],
            );
            let scale = Mat4::from_nonuniform_scale((p1 - p2).magnitude(), 1.0, 1.0);
            let rotation =
                rotation_matrix_from_dir_to_dir(vec3(1.0, 0.0, 0.0), (p2 - p1).normalize());
            let translation = Mat4::from_translation(p1);
            let key = if i1 < i2 { (i1, i2) } else { (i2, i1) };
            edge_transformations.insert(key, translation * rotation * scale);
        };
        let i1 = indices[3 * f] as usize;
        let i2 = indices[3 * f + 1] as usize;
        let i3 = indices[3 * f + 2] as usize;
        fun(i1, i2);
        fun(i1, i3);
        fun(i2, i3);
    }
    edge_transformations
        .drain()
        .map(|(_, v)| v)
        .collect::<Vec<Mat4>>()
}

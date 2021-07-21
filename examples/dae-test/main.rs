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
            "./examples/assets/sphere_03.dae",
            "./examples/assets/sphere_02.dae",
            "./examples/assets/cube_01.dae",
            "./examples/assets/cube_02.dae",
            "./examples/assets/sphere.jpg",
            "./examples/assets/chips.jpg",
        ],
        move |mut loaded| {
            let (cpu_meshes, cpu_materials) = loaded.dae("./examples/assets/sphere_03.dae").unwrap();
            let mut model = Mesh::new(
                &context,
                &cpu_meshes[0],
                // &Material::new(&context, &cpu_materials[0]).unwrap(),
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

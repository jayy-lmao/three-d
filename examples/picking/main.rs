use three_d::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let window = Window::new(WindowSettings {
        title: "Picking!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl().unwrap();

    let target = vec3(0.0, 0.0, 0.0);
    let mut camera = CameraControl::new(
        Camera::new_perspective(
            &context,
            window.viewport().unwrap(),
            vec3(4.0, 4.0, 5.0),
            target,
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        )
        .unwrap(),
        EventHandler {
            left_drag: ControlType::RotateAroundWithFixedUp { target, speed: 0.1 },
            scroll: ControlType::ZoomOnVertical {
                target,
                speed: 0.02,
                min: 5.0,
                max: 100.0,
            },
            ..Default::default()
        },
    );

    let mut pick_mesh = Mesh::new_with_material(
        &context,
        &CPUMesh::sphere(0.05),
        &Material::new(
            &context,
            &CPUMaterial {
                color: Some((1.0, 0.0, 0.0, 1.0)),
                ..Default::default()
            },
        )
        .unwrap(),
    )
    .unwrap();

    Loader::load(
        &["examples/assets/suzanne.obj", "examples/assets/suzanne.mtl"],
        move |loaded| {
            let (meshes, mut materials) = loaded.obj("examples/assets/suzanne.obj").unwrap();
            materials[0].color = Some((0.5, 1.0, 0.5, 1.0));
            let mut monkey = Mesh::new_with_material(
                &context,
                &meshes[0],
                &Material::new(&context, &materials[0]).unwrap(),
            )
            .unwrap();
            monkey.cull = CullType::Back;

            let ambient_light = AmbientLight {
                intensity: 0.4,
                color: vec3(1.0, 1.0, 1.0),
            };
            let directional_light =
                DirectionalLight::new(&context, 2.0, &vec3(1.0, 1.0, 1.0), &vec3(-1.0, -1.0, -1.0))
                    .unwrap();

            // main loop
            window
                .render_loop(move |mut frame_input| {
                    let mut change = frame_input.first_frame;
                    change |= camera.set_viewport(frame_input.viewport).unwrap();

                    for event in frame_input.events.iter_mut() {
                        match event {
                            Event::MouseClick {
                                state,
                                button,
                                position,
                                handled,
                                ..
                            } => {
                                if *button == MouseButton::Left && *state == State::Pressed {
                                    let pixel = (
                                        (frame_input.device_pixel_ratio * position.0) as f32,
                                        (frame_input.device_pixel_ratio * position.1) as f32,
                                    );
                                    if let Some(pick) =
                                        camera.pick(pixel, 100.0, &[&monkey]).unwrap()
                                    {
                                        pick_mesh.transformation = Mat4::from_translation(pick);
                                        change = true;
                                        *handled = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    change |= camera.handle_events(&frame_input.events).unwrap();

                    // draw
                    if change {
                        Screen::write(
                            &context,
                            ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0),
                            || {
                                monkey.render_with_lighting(
                                    RenderStates {
                                        depth_test: DepthTestType::LessOrEqual,
                                        ..Default::default()
                                    },
                                    &camera,
                                    Some(&ambient_light),
                                    &[&directional_light],
                                    &[],
                                    &[],
                                )?;
                                pick_mesh.render_with_lighting(
                                    RenderStates::default(),
                                    &camera,
                                    Some(&ambient_light),
                                    &[&directional_light],
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
                            swap_buffers: change,
                            ..Default::default()
                        }
                    }
                })
                .unwrap();
        },
    );
}

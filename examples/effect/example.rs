
use three_d::*;
use std::cell::RefCell;
use std::rc::Rc;

use log::info;

struct Pars {
    renderer: Rc<RefCell<DeferredPipeline>>,
    gl: Gl,
    camera: Camera,
    monkey: Mesh,
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    fog_effect: effects::FogEffect,
    fxaa_effect: effects::FXAAEffect,
    time: f64,
    rotating: bool,
    fog_enabled: bool,
    fxaa_enabled: bool,
    screenshot_path: Option<String>,
    model_load_path: Option<String>
}

async fn run() {
    let args: Vec<String> = std::env::args().collect();
    let screenshot_path = if args.len() > 1 { Some(args[1].clone()) } else {None};

    let mut window = Window::new_default("Effect").unwrap();
    let gl = window.gl();

    // Renderer
    let camera = Camera::new_perspective(&gl, vec3(4.0, 4.0, 5.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0),
                                                degrees(45.0), 1.0, 0.1, 1000.0);

    let ambient_light = AmbientLight::new(&gl, 0.2, &vec3(1.0, 1.0, 1.0)).unwrap();
    let directional_light = DirectionalLight::new(&gl, 0.5, &vec3(1.0, 1.0, 1.0), &vec3(-1.0, -1.0, -1.0)).unwrap();

    let mut monkey = CPUMesh::from_file("./examples/assets/models/suzanne.3d").await.unwrap().to_mesh(&gl).unwrap();
    monkey.color = vec3(0.5, 1.0, 0.5);

    let mut fog_effect = effects::FogEffect::new(&gl).unwrap();
    fog_effect.color = vec3(0.8, 0.8, 0.8);
    let fxaa_effect = effects::FXAAEffect::new(&gl).unwrap();

    let pars = Rc::new(RefCell::new(Pars {
        renderer: Rc::new(RefCell::new(DeferredPipeline::new(&gl).unwrap())),
        gl,
        camera,
        monkey,
        ambient_light,
        directional_light,
        fog_effect,
        fxaa_effect,
        time: 0.0,
        rotating: false,
        fxaa_enabled: true,
        fog_enabled: true,
        screenshot_path,
        model_load_path: Some("./examples/assets/models/suzanne.3d".to_string())
    }));

    let callback = |pars: Rc<RefCell<Pars>>, frame_input: FrameInput| -> bool
    {
        pars.borrow_mut().camera.set_size(frame_input.screen_width as f32, frame_input.screen_height as f32);

        for event in frame_input.events.iter() {
            match event {
                Event::MouseClick {state, button, ..} => {
                    pars.borrow_mut().rotating = *button == MouseButton::Left && *state == State::Pressed;
                },
                Event::MouseMotion {delta} => {
                    if pars.borrow().rotating {
                        pars.borrow_mut().camera.rotate(delta.0 as f32, delta.1 as f32);
                    }
                },
                Event::MouseWheel {delta} => {
                    pars.borrow_mut().camera.zoom(*delta as f32);
                },
                Event::Key { state, kind } => {
                    if kind == "Escape" && *state == State::Pressed
                    {
                        return true;
                    }
                    if kind == "R" && *state == State::Pressed
                    {
                        pars.borrow().renderer.borrow_mut().next_debug_type();
                        println!("{:?}", pars.borrow().renderer.borrow().debug_type());
                    }
                    if kind == "F" && *state == State::Pressed
                    {
                        pars.borrow_mut().fog_enabled = !pars.borrow().fog_enabled;
                        println!("Fog: {:?}", pars.borrow().fog_enabled);
                    }
                    if kind == "X" && *state == State::Pressed
                    {
                        pars.borrow_mut().fxaa_enabled = !pars.borrow().fxaa_enabled;
                        println!("FXAA: {:?}", pars.borrow().fxaa_enabled);
                    }
                    if kind == "L" && *state == State::Pressed
                    {
                        pars.borrow_mut().model_load_path = Some("./examples/assets/models/tree1.3d".to_string());
                        println!("FXAA: {:?}", pars.borrow().model_load_path);
                        return true;
                    }
                }
            }
        }
        pars.borrow_mut().time += frame_input.elapsed_time;

        // draw
        pars.borrow().renderer.borrow_mut().geometry_pass(frame_input.screen_width, frame_input.screen_height, &|| {
            let transformation = Mat4::identity();
            pars.borrow().monkey.render(&transformation, &pars.borrow().camera);
        }).unwrap();

        let render = || {
                pars.borrow().renderer.borrow_mut().light_pass(&pars.borrow().camera, Some(&pars.borrow().ambient_light), &[&pars.borrow().directional_light], &[], &[]).unwrap();
                if pars.borrow().fog_enabled {
                    pars.borrow().fog_effect.apply(pars.borrow().time as f32, &pars.borrow().camera, pars.borrow().renderer.borrow().geometry_pass_depth_texture()).unwrap();
                }
            };

        if pars.borrow().fxaa_enabled {
            let color_texture = Texture2D::new(&pars.borrow().gl, frame_input.screen_width, frame_input.screen_height, Interpolation::Nearest,
                         Interpolation::Nearest, None, Wrapping::ClampToEdge, Wrapping::ClampToEdge, Format::RGBA8).unwrap();
            RenderTarget::write_to_color(&pars.borrow().gl,0, 0, frame_input.screen_width, frame_input.screen_height,Some(&vec4(0.0, 0.0, 0.0, 0.0)),
                                         Some(&color_texture), &render).unwrap();
            Screen::write(&pars.borrow().gl, 0, 0, frame_input.screen_width, frame_input.screen_height, Some(&vec4(0.0, 0.0, 0.0, 1.0)), None, &|| {
                pars.borrow().fxaa_effect.apply(&color_texture).unwrap();
            }).unwrap();
        } else {
            Screen::write(&pars.borrow().gl, 0, 0, frame_input.screen_width, frame_input.screen_height, Some(&vec4(0.0, 0.0, 0.0, 1.0)), None, &render).unwrap();
        }

        if let Some(ref path) = pars.borrow().screenshot_path {
            #[cfg(target_arch = "x86_64")]
            Screen::save_color(path, &pars.borrow().gl, 0, 0, frame_input.screen_width, frame_input.screen_height).unwrap();
            return true;
        }
        false
    };

    while pars.borrow().model_load_path.is_some() {
        info!("Load");
        let load_path = pars.borrow().model_load_path.clone().unwrap();
        pars.borrow_mut().monkey = CPUMesh::from_file(&load_path).await.unwrap().to_mesh(&window.gl()).unwrap();
        pars.borrow_mut().model_load_path = None;
        pars.borrow_mut().monkey.color = vec3(0.5, 1.0, 0.5);
        info!("Start");
        window.render_loop_with_parameters(pars.clone(), callback).unwrap();

        info!("Stop");
    }
}
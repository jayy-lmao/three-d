use crate::camera::*;
use crate::core::*;
use crate::frame::*;
use crate::math::*;

pub struct EventHandler {
    pub left_drag: ControlType,
    pub middle_drag: ControlType,
    pub right_drag: ControlType,
    pub scroll: ControlType,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self {
            left_drag: ControlType::None,
            middle_drag: ControlType::None,
            right_drag: ControlType::None,
            scroll: ControlType::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlType {
    None,
    RotateAround {
        target: Vec3,
        speed: f32,
    },
    RotateAroundWithFixedUp {
        target: Vec3,
        speed: f32,
    },
    Pan {
        speed: f32,
    },
    ZoomHorizontal {
        target: Vec3,
        speed: f32,
        min: f32,
        max: f32,
    },
    ZoomVertical {
        target: Vec3,
        speed: f32,
        min: f32,
        max: f32,
    },
}

///
/// 3D controls for a camera. Use this to add additional control functionality to a [camera](crate::Camera).
///
pub struct CameraControl {
    camera: Camera,
    left: bool,
    middle: bool,
    right: bool,
    pub event_handler: EventHandler,
}

impl CameraControl {
    pub fn new(camera: Camera, event_handler: EventHandler) -> Self {
        Self {
            camera,
            left: false,
            middle: false,
            right: false,
            event_handler,
        }
    }

    pub fn handle_events(&mut self, events: &Vec<Event>) -> Result<bool, Error> {
        let mut change = false;
        for event in events.iter() {
            match event {
                Event::MouseClick {
                    button,
                    handled,
                    state,
                    ..
                } => {
                    if !*handled {
                        match *button {
                            MouseButton::Left => {
                                self.left = *state == State::Pressed;
                            }
                            MouseButton::Middle => {
                                self.middle = *state == State::Pressed;
                            }
                            MouseButton::Right => {
                                self.right = *state == State::Pressed;
                            }
                        }
                    }
                }
                Event::MouseMotion {
                    delta: (x, y),
                    handled,
                    ..
                } => {
                    if !*handled {
                        if self.left {
                            change |= self.handle_drag(self.event_handler.left_drag, *x, *y)?;
                        }
                        if self.middle {
                            change |= self.handle_drag(self.event_handler.middle_drag, *x, *y)?;
                        }
                        if self.right {
                            change |= self.handle_drag(self.event_handler.right_drag, *x, *y)?;
                        }
                    }
                }
                Event::MouseWheel {
                    delta: (x, y),
                    handled,
                    ..
                } => {
                    if !*handled {
                        change |= self.handle_drag(self.event_handler.scroll, *x, *y)?;
                    }
                }
                _ => {}
            }
        }
        Ok(change)
    }

    fn handle_drag(&mut self, control_type: ControlType, x: f64, y: f64) -> Result<bool, Error> {
        match control_type {
            ControlType::RotateAround { speed, target } => {
                self.rotate_around(&target, speed * x as f32, speed * y as f32)?;
            }
            ControlType::RotateAroundWithFixedUp { speed, target } => {
                self.rotate_around_with_fixed_up(&target, speed * x as f32, speed * y as f32)?;
            }
            ControlType::Pan { speed } => {
                self.pan(speed * x as f32, speed * y as f32)?;
            }
            ControlType::ZoomHorizontal {
                target,
                speed,
                min,
                max,
            } => {
                self.zoom_towards(&target, speed * x as f32, min, max)?;
            }
            ControlType::ZoomVertical {
                target,
                speed,
                min,
                max,
            } => {
                self.zoom_towards(&target, speed * y as f32, min, max)?;
            }
            ControlType::None => {}
        }
        Ok(control_type != ControlType::None)
    }
}

impl std::ops::Deref for CameraControl {
    type Target = Camera;

    fn deref(&self) -> &Self::Target {
        &self.camera
    }
}

impl std::ops::DerefMut for CameraControl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.camera
    }
}

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

    ///
    /// Translate the camera by the given change while keeping the same view and up directions.
    ///
    pub fn translate(&mut self, change: &Vec3) -> Result<(), Error> {
        let position = *self.position();
        let target = *self.target();
        let up = *self.up();
        self.set_view(position + change, target + change, up)?;
        Ok(())
    }

    ///
    /// Rotate the camera around the given point while keeping the same distance to the point.
    /// The input `x` specifies the amount of rotation in the left direction and `y` specifies the amount of rotation in the up direction.
    /// If you want the camera up direction to stay fixed, use the [rotate_around_with_fixed_up](crate::CameraControl::rotate_around_with_fixed_up) function instead.
    ///
    pub fn rotate_around(&mut self, point: &Vec3, x: f32, y: f32) -> Result<(), Error> {
        let dir = (point - self.position()).normalize();
        let right = dir.cross(*self.up());
        let up = right.cross(dir);
        let new_dir = (point - self.position() + right * x - up * y).normalize();
        let rotation = rotation_matrix_from_dir_to_dir(dir, new_dir);
        let new_position = (rotation * (self.position() - point).extend(1.0)).truncate() + point;
        let new_target = (rotation * (self.target() - point).extend(1.0)).truncate() + point;
        self.set_view(new_position, new_target, up)?;
        Ok(())
    }

    ///
    /// Rotate the camera around the given point while keeping the same distance to the point and the same up direction.
    /// The input `x` specifies the amount of rotation in the left direction and `y` specifies the amount of rotation in the up direction.
    ///
    pub fn rotate_around_with_fixed_up(
        &mut self,
        point: &Vec3,
        x: f32,
        y: f32,
    ) -> Result<(), Error> {
        let dir = (point - self.position()).normalize();
        let right = dir.cross(*self.up());
        let mut up = right.cross(dir);
        let new_dir = (point - self.position() + right * x - up * y).normalize();
        up = *self.up();
        if new_dir.dot(up).abs() < 0.999 {
            let rotation = rotation_matrix_from_dir_to_dir(dir, new_dir);
            let new_position =
                (rotation * (self.position() - point).extend(1.0)).truncate() + point;
            let new_target = (rotation * (self.target() - point).extend(1.0)).truncate() + point;
            self.set_view(new_position, new_target, up)?;
        }
        Ok(())
    }

    ///
    /// Moves the camera in the plane orthogonal to the current view direction, which means the view and up directions will stay the same.
    /// The input `x` specifies the amount of translation in the left direction and `y` specifies the amount of translation in the up direction.
    ///
    pub fn pan(&mut self, x: f32, y: f32) -> Result<(), Error> {
        let right = self.right_direction();
        let up = right.cross(self.view_direction());
        let delta = -right * x + up * y;
        self.translate(&delta)?;
        Ok(())
    }

    ///
    /// Moves the camera towards the given point by the amount delta while keeping the given minimum and maximum distance to the point.
    ///
    pub fn zoom_towards(
        &mut self,
        point: &Vec3,
        delta: f32,
        minimum_distance: f32,
        maximum_distance: f32,
    ) -> Result<(), Error> {
        if minimum_distance <= 0.0 {
            return Err(Error::CameraError {
                message: "Zoom towards cannot take as input a negative minimum distance."
                    .to_string(),
            });
        }
        if maximum_distance < minimum_distance {
            return Err(Error::CameraError {
                message: "Zoom towards cannot take as input a maximum distance which is smaller than the minimum distance."
                    .to_string(),
            });
        }
        let position = *self.position();
        let distance = point.distance(position);
        let direction = (point - position).normalize();
        let target = *self.target();
        let up = *self.up();
        let new_distance = (distance - delta)
            .max(minimum_distance)
            .min(maximum_distance);
        let new_position = point - direction * new_distance;
        self.set_view(new_position, new_position + (target - position), up)?;
        match self.projection_type() {
            ProjectionType::Orthographic { width: _, height } => {
                let h = new_distance * height / distance;
                let z_near = self.z_near();
                let z_far = self.z_far();
                self.set_orthographic_projection(h, z_near, z_far)?;
            }
            _ => {}
        }
        Ok(())
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

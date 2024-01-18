use bevy::prelude::*;

#[derive(Component)]
pub struct Updated(pub(crate) bool);

#[repr(C)]
#[derive(Copy, Clone, Debug, Component)]
pub struct Drawable {
    pub position: [f32; 3],
    pub shape_data: [f32; 8],
    pub shape: u32,
}

impl Drawable {

    pub fn new(x: isize, y: isize, z: isize, shape: Option<u32>) -> Self {
        let shape = shape.unwrap_or(0);
        Drawable {
            position: [x as f32, y as f32, z as f32],
            shape_data: [0.0; 8],
            shape,
            ..default()
        }
    }
    pub(crate) fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Default for Drawable {
    fn default() -> Self {
        Drawable {
            position: [0.0, 0.0, 0.0],
            shape_data: [0.0; 8],
            shape: 0,
        }
    }
}

unsafe impl bytemuck::Zeroable for Drawable {}
unsafe impl bytemuck::Pod for Drawable {}
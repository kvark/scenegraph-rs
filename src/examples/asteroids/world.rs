#[phase(plugin)]
extern crate gfx_macros;

use cgmath::{Rad, Basis2, Rotation, Rotation2, Point2, Vector2};
use gfx;
use scenegraph::ces;

pub type Delta = f32;
pub type Params<'a, 'b> = &'a mut (Delta, &'b mut gfx::DrawList);

#[shader_param(Program)]
pub struct ShaderParam {
	//TODO: hide these
	pub transform: [f32, ..4],
	pub screen_scale: [f32, ..4],
}

/// --- Components ---

#[deriving(Clone)]
pub struct Drawable {
	pub program: Program,
	pub mesh_id: ces::Id<gfx::Mesh>,
	pub state_id: ces::Id<gfx::DrawState>,
	pub slice: gfx::Slice,
}

#[deriving(Clone)]
pub struct Spatial {
	pub pos: Point2<f32>,
	pub orient: Rad<f32>,
	pub scale: f32,
}

impl Spatial {
	pub fn get_direction(&self) -> Vector2<f32> {
		let rot: Basis2<f32> = Rotation2::from_angle(self.orient);
		rot.rotate_vector(&Vector2::unit_y())
	}
}

#[deriving(Clone)]
pub struct Inertial {
	pub velocity: Vector2<f32>,
	pub angular_velocity: Rad<f32>,
}

#[deriving(Clone)]
pub struct Control {
	pub thrust_speed: f32,
	pub turn_speed: f32,
}

#[deriving(Clone)]
pub struct Bullet {
	pub life_time: Option<f32>,
}


world! { ces (Params),
	draw: Drawable,
	space: Spatial,
	inertia: Inertial,
	control: Control,
	bullet: Bullet,
}

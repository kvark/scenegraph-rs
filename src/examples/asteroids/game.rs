extern crate time;

use cgmath::{Rad, Point2, Vector2};
use gl_init;
use gfx;
use gfx::DeviceHelper;
use sys;
use world;

pub type EventReceiver = (
	Receiver<sys::control::Event>,
	Receiver<sys::bullet::Event>
);

pub struct EventSender {
	control: Sender<sys::control::Event>,
	bullet: Sender<sys::bullet::Event>,
}

impl EventSender {
	pub fn new() -> (EventSender, EventReceiver) {
		let (sc, rc) = channel();
		let (sb, rb) = channel();
		(EventSender {
			control: sc,
			bullet: sb,
		}, (rc, rb))
	}

	pub fn process(&self, event: gl_init::Event) {
		use sys::control::{EvThrust, EvTurn};
		use sys::bullet::{EvShoot};
		match event {
			gl_init::KeyboardInput(state, _, Some(gl_init::A), _) =>
				self.control.send(EvThrust(match state {
					gl_init::Pressed => 1.0,
					gl_init::Released => 0.0,
				})),
			gl_init::KeyboardInput(gl_init::Pressed, _, Some(gl_init::Left), _) =>
				self.control.send(EvTurn(-1.0)),
			gl_init::KeyboardInput(gl_init::Pressed, _, Some(gl_init::Right), _) =>
				self.control.send(EvTurn(1.0)),
			gl_init::KeyboardInput(gl_init::Released, _, Some(k), _)
				if k == gl_init::Left || k == gl_init::Right =>
				self.control.send(EvTurn(0.0)),
			gl_init::KeyboardInput(state, _, Some(gl_init::S), _) =>
				self.bullet.send(EvShoot(match state {
					gl_init::Pressed => true,
					gl_init::Released => false,
				})),
			_ => (),
		}
	}
}

#[vertex_format]
struct Vertex {
	pos: [f32, ..2],
	#[normalized]
	color: [u8, ..4],
}

impl Vertex {
	fn new(x: f32, y: f32, col: uint) -> Vertex {
		Vertex {
			pos: [x, y],
			color: [(col>>24) as u8, (col>>16) as u8, (col>>8) as u8, col as u8],
		}
	}
}

pub struct Game {
	world: world::World,
	last_time: u64,
}

impl Game {
	fn create_program<T, D: gfx::Device<T>>(device: &mut D) -> world::Program {
		device.link_program(
			world::ShaderParam {
				transform: [0.0, 0.0, 0.0, 1.0],
				screen_scale: [0.1, 0.1, 0.0, 0.0],
			},
			shaders! {
			GLSL_120: b"
				#version 120
				attribute vec2 pos;
				attribute vec4 color;
				uniform vec4 transform, screen_scale;
				varying vec4 v_color;
				void main() {
					v_color = color;
					vec2 sc = vec2(sin(transform.z), cos(transform.z));
					vec2 p = vec2(pos.x*sc.y - pos.y*sc.x, pos.x*sc.x + pos.y*sc.y);
					p = (p * transform.w + transform.xy) * screen_scale.xy;
					gl_Position = vec4(p, 0.0, 1.0);
				}
			"},
			shaders! {
			GLSL_120: b"
				#version 120
				varying vec4 v_color;
				void main() {
					gl_FragColor = v_color;
				}
			"}
		).unwrap()
	}

	fn create_ship<T, D: gfx::Device<T>>(device: &mut D, data: &mut world::Components,
				   draw: &mut sys::draw::System, program: world::Program)
				   -> world::Entity {
		let mesh = device.create_mesh(vec![
			Vertex::new(-0.3, -0.5, 0x20C02000),
			Vertex::new(0.3, -0.5,  0x20C02000),
			Vertex::new(0.0, 0.5,   0xC0404000),
		]);
		let slice = mesh.get_slice();
		let mut state = gfx::DrawState::new();
		state.primitive.method = gfx::state::Fill(gfx::state::CullNothing);
		data.add()
			.draw(world::Drawable {
				program: program,
				mesh_id: draw.meshes.add(mesh),
				state_id: draw.states.add(state),
				slice: slice,
			})
			.space(world::Spatial {
				pos: Point2::new(0.0, 0.0),
				orient: Rad{ s: 0.0 },
				scale: 1.0,
			})
			.inertia(world::Inertial {
				velocity: Vector2::zero(),
				angular_velocity: Rad{ s:0.0 },
			})
			.control(world::Control {
				thrust_speed: 4.0,
				turn_speed: -90.0,
			})
			.entity
	}

	pub fn new<T, D: gfx::Device<T>>(frame: gfx::Frame,
			   (ev_control, ev_bullet): EventReceiver, device: &mut D) -> Game {
		let mut w = world::World::new();
		// prepare systems
		let program = Game::create_program(device);
		let mut draw_system = sys::draw::System::new(frame);
		let bullet_draw = {
			let mut mesh = device.create_mesh(vec![
				Vertex::new(0.0, 0.0, 0xFF404000),
			]);
			mesh.prim_type = gfx::Point;
			let slice = mesh.get_slice();
			let mut state = gfx::DrawState::new();
			state.primitive.method = gfx::state::Point;
			world::Drawable {
				program: program.clone(),
				mesh_id: draw_system.meshes.add(mesh),
				state_id: draw_system.states.add(state),
				slice: slice,
			}
		};
		let ship = Game::create_ship(device, &mut w.data, &mut draw_system, program);
		let (space_id, inertia_id) = (ship.space.unwrap(), ship.inertia.unwrap());
		// populate world and return
		w.entities.push(ship);
		w.systems.push_all_move(vec![
			box draw_system as Box<world::System + Send>,
			box sys::inertia::System,
			box sys::control::System::new(ev_control),
			box sys::bullet::System::new(ev_bullet,
				space_id, inertia_id, bullet_draw),
		]);
		Game {
			world: w,
			last_time: time::precise_time_ns(),
		}
	}

	pub fn render(&mut self, list: &mut gfx::DrawList) {
		let new_time = time::precise_time_ns();
		let delta = (new_time - self.last_time) as f32 / 1e9;
		self.last_time = new_time;
		self.world.update(&mut (delta, list));
	}
}

use glium;
use image;

use filesystem;
use resourcemanager;
use camera::Camera;

use std::io::Cursor;
use std::sync::{Arc, Mutex, Once, ONCE_INIT};
use std::{mem, thread};
use std::time::Duration;

use glium::glutin;

#[derive(Clone)]
pub struct RenderManager {
	// Since we will be used in many threads, we need to protect
	// concurrent access
	pub inner: Arc<Mutex<u8>>,
	pub sleep_time: Arc<Mutex<u64>>,
}

#[derive(Copy, Clone)]
struct Vertex {
	position: [f32; 3],
	normal: [f32; 3],
	tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coords);

use scenemanager::SceneManager;
impl RenderManager {
	pub fn start_up(&mut self) {
		SceneManager::instance().add_camera(Camera::new());

		self.main_loop();
	}

	#[allow(dead_code)]
	pub fn shout_down() {

	}

	pub fn instance() -> RenderManager {
		// Initialize it to a null value
		static mut SINGLETON: *const RenderManager = 0 as *const RenderManager;
		static ONCE: Once = ONCE_INIT;

		unsafe {
			ONCE.call_once(|| {
				// Make it
				let instance = RenderManager {
					inner: Arc::new(Mutex::new(0)),
					sleep_time:Arc::new(Mutex::new(0u64)),
				};

				// Put it in the heap so it can outlive this call
				SINGLETON = mem::transmute(Box::new(instance));
			});

			// Now we give out a copy of the data that is safe to use concurrently.
			(*SINGLETON).clone()
		}
	}

	pub fn render(&mut self, elapsed_time:u64)
	{
		*self.sleep_time.lock().unwrap() = (1000.0 / 60.0) as u64 - elapsed_time;
	}

	#[allow(dead_code)]
	fn pause()
	{

	}

	fn main_loop(&mut self) {

		use glium::{DisplayBuild, Surface};
		let display = glium::glutin::WindowBuilder::new()
						.with_depth_buffer(24)
						.build_glium().unwrap();

		let vertex_buffer = resourcemanager::load_wavefront(&display, include_bytes!("../artist/Objects/teapot.obj"));

		let shape = glium::vertex::VertexBuffer::new(&display, &[
				Vertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0] },
				Vertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0] },
				Vertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0] },
				Vertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0] },
			]).unwrap();


		let image = image::load(Cursor::new(&include_bytes!("../test/tuto-14-diffuse.jpg")[..]),
								image::JPEG).unwrap().to_rgba();
		let image_dimensions = image.dimensions();
		let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
		let diffuse_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

		let image = image::load(Cursor::new(&include_bytes!("../test/tuto-14-normal.png")[..]),
								image::PNG).unwrap().to_rgba();
		let image_dimensions = image.dimensions();
		let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
		let normal_map = glium::texture::Texture2d::new(&display, image).unwrap();


		let vertex_shader_src = filesystem::open("./test/vertex_shader");
		let vertex_shader_src_slice: &str = &vertex_shader_src;
		let fragment_shader_src = filesystem::open("./test/fragment_shader");
		let fragment_shader_src_slice: &str = &fragment_shader_src;

		let program = glium::Program::from_source(&display, vertex_shader_src_slice, fragment_shader_src_slice,
												  None).unwrap();

		let t_vertex_shader_src = filesystem::open("./test/teapot_vertex_shader");
		let t_vertex_shader_src_slice: &str = &t_vertex_shader_src;
		let t_fragment_shader_src = filesystem::open("./test/teapot_fragment_shader");
		let t_fragment_shader_src_slice: &str = &t_fragment_shader_src;

		let t_program = glium::Program::from_source(&display, t_vertex_shader_src_slice, t_fragment_shader_src_slice,
												  None).unwrap();

		let mut t:f32 = -0.5;

		loop {
            let mut camera: Camera = SceneManager::instance().get_camera(0);
			camera.update();

			let mut target = display.draw();
			target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

			t += 0.00001;
			if t > 0.5 {
				t = -0.5;
			}
			let model = [
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[t, 0.0, 0.0, 1.0f32]
			];

			let light = [1.4, 0.4, 0.7f32];

			let params = glium::DrawParameters {
				depth: glium::Depth {
					test: glium::draw_parameters::DepthTest::IfLess,
					write: true,
					.. Default::default()
				},
				.. Default::default()
			};

			target.draw(&vertex_buffer,
						&glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
						&t_program, &uniform! { model: model, view: camera.get_view(), perspective: camera.get_perspective(),
									u_light: light, diffuse_tex: &diffuse_texture, normal_tex: &normal_map },
						&params).unwrap();

			target.draw(&shape,
						glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
						&program, &uniform! { model: model, view: camera.get_view(), perspective: camera.get_perspective(),
									u_light: light, diffuse_tex: &diffuse_texture, normal_tex: &normal_map },
						&params).unwrap();

			target.finish().unwrap();


			thread::sleep(Duration::from_millis(*self.sleep_time.lock().unwrap()));

			for ev in display.poll_events() {
				match ev {
					glium::glutin::Event::Closed => return,
                    ev => self.process_input(&ev),
				}
			}
		}
	}

    pub fn process_input(&mut self, event: &glutin::Event) {
		let mut camera = SceneManager::instance().get_camera(0);
        camera.process_input(event);
    }
}

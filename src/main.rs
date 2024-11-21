extern crate gl;
extern crate sdl2;
#[macro_use] extern crate render_gl_derive;

const WINDOW_TITLE: &str = "Test window";

pub mod render_gl;
pub mod resources;

use crate::render_gl::data;
use resources::Resources;
use std::path::Path;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
	#[location(0)]
	pos: data::Coordinate,
	#[location(1)]
	clr: data::Coordinate,
}


fn main() {
	let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

	let sdl = sdl2::init().unwrap();
	let video_subsystem = sdl.video().unwrap();
	let window = video_subsystem
		.window(WINDOW_TITLE, 900, 900)
		.opengl()
		.resizable()
		.build()
		.unwrap();
	let _gl_context = window.gl_create_context().unwrap();
	let gl = gl::Gl::load_with(|s| {
		video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
	});
	let gl_attr = video_subsystem.gl_attr();
	gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
	gl_attr.set_context_version(4, 5);

	let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/triangle").unwrap();
	shader_program.set_used();

	let vertices: Vec<Vertex> = vec![
		// Location      Color
		Vertex {
			pos: (-0.5, -0.5, 0.0).into(),
			clr: (1.0, 0.0, 0.0).into(),
		}, // bottom right
		Vertex {
			pos: (0.5, -0.5, 0.0).into(),
			clr: (0.0, 1.0, 0.0).into(),
		}, // bottom left
		Vertex {
			pos: (0.0, 0.5, 0.0).into(),
			clr: (0.0, 0.0, 1.0).into(),
		}, // top
	];
	let mut vbo: gl::types::GLuint = 0;
	unsafe {
		gl.GenBuffers(1, &mut vbo);
	};
	unsafe {
		gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
		// We have to provide a pointer to the array which will be overwritten with a new value.
		// Since rust references (`&` and `&mut`) are pointers, we can simply pass the along.
		// However, we must limit the number of buffers to `1` so we do not overwrite unknown memory nearby.
		gl.BufferData(
			gl::ARRAY_BUFFER,                                                          // target
			(vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, // size of data (bytes)
			vertices.as_ptr() as *const gl::types::GLvoid,                             // pointer to data
			gl::STATIC_DRAW,                                                           // usage
		);
		gl.BindBuffer(gl::ARRAY_BUFFER, 0); // unbind buffer
	}

	let mut vao: gl::types::GLuint = 0;
	unsafe {
		gl.GenVertexArrays(1, &mut vao);
	}
	unsafe {
		gl.BindVertexArray(vao);
		gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
		Vertex::vertex_attrib_pointers(&gl);
		gl.BindBuffer(gl::ARRAY_BUFFER, 0);
		gl.BindVertexArray(0);
	}

	unsafe {
		gl.Viewport(0, 0, 900, 700);
		gl.ClearColor(0.3, 0.3, 0.5, 1.0);
	}

	let mut event_pump = sdl.event_pump().unwrap();
	'main_loop: loop {
		for event in event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit { .. } => break 'main_loop,
				_ => {}
			}
		}
		unsafe {
			gl.Clear(gl::COLOR_BUFFER_BIT);
		}

		// draw triangle
		shader_program.set_used();
		unsafe {
			gl.BindVertexArray(vao);
			gl.DrawArrays(
				gl::TRIANGLES, // mode
				0,             // starting index in the enabled arrays
				3,             // number of indices to render
			);
		}
		window.gl_swap_window();
	}
}

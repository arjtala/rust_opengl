use gl;
use std;
use std::ffi::{CStr, CString};

use crate::resources::{ResourceError, Resources};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderError {
	#[error("Unable to load resource")]
	LoadResourceError { name: String, inner: ResourceError },
	#[error("Cannot determine type for resource")]
	TypeError { name: String },
	#[error("Cannot compile")]
	CompileError { name: String, message: String },
	#[error("Unable to link")]
	LinkError,
}

pub struct Program {
	gl: gl::Gl,
	id: gl::types::GLuint,
}

impl Program {
	pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, ShaderError> {
		let program_id = unsafe { gl.CreateProgram() };
		for shader in shaders {
			unsafe {
				gl.AttachShader(program_id, shader.id());
			}
		}
		unsafe {
			gl.LinkProgram(program_id);
		}

		// continue with error handling here
		let mut success: gl::types::GLint = 1;
		unsafe {
			gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
		}
		if success == 0 {
			let mut len: gl::types::GLint = 0;
			unsafe {
				gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
			}
			let error = create_whitespace_cstring_with_len(len as usize);
			unsafe {
				gl.GetProgramInfoLog(
					program_id,
					len,
					std::ptr::null_mut(),
					error.as_ptr() as *mut gl::types::GLchar,
				);
			}
			return Err(ShaderError::LinkError);
		}

		for shader in shaders {
			unsafe {
				gl.DetachShader(program_id, shader.id());
			}
		}
		Ok(Program {
			gl: gl.clone(),
			id: program_id,
		})
	}

	pub fn id(&self) -> gl::types::GLuint {
		self.id
	}

	pub fn set_used(&self) {
		unsafe {
			self.gl.UseProgram(self.id);
		}
	}

	pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, ShaderError> {
		const POSSIBLE_EXT: [&str; 2] = [".vert", ".frag"];
		let resource_names = POSSIBLE_EXT
			.iter()
			.map(|file_extension| format!("{}{}", name, file_extension))
			.collect::<Vec<String>>();

		let shaders = resource_names
			.iter()
			.map(|resource_name| Shader::from_res(gl, res, resource_name))
			.collect::<Result<Vec<Shader>, ShaderError>>()?;

		Program::from_shaders(gl, &shaders[..])
	}
}

impl Drop for Program {
	fn drop(&mut self) {
		unsafe {
			self.gl.DeleteProgram(self.id);
		}
	}
}

pub struct Shader {
	gl: gl::Gl,
	id: gl::types::GLuint,
}

impl Shader {
	pub fn from_source(
		gl: &gl::Gl,
		source: &CStr,
		kind: gl::types::GLenum,
	) -> Result<Shader, String> {
		let id = shader_from_source(gl, source, kind)?;
		Ok(Shader { gl: gl.clone(), id })
	}

	pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, ShaderError> {
		const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] =
			[(".vert", gl::VERTEX_SHADER), (".frag", gl::FRAGMENT_SHADER)];
		let shader_kind = POSSIBLE_EXT
			.iter()
			.find(|&&(file_extension, _)| name.ends_with(file_extension))
			.map(|&(_, kind)| kind)
			.ok_or_else(|| ShaderError::TypeError { name: name.into() })?;
		let source = res
			.load_cstring(name)
			.map_err(|e| ShaderError::LoadResourceError {
				name: name.into(),
				inner: e,
			})?;
		Shader::from_source(gl, &source, shader_kind).map_err(|message| ShaderError::CompileError {
			name: name.into(),
			message,
		})
	}

	pub fn from_vert_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
		Shader::from_source(gl, source, gl::VERTEX_SHADER)
	}

	pub fn from_frag_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
		Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
	}

	pub fn id(&self) -> gl::types::GLuint {
		self.id
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		unsafe {
			self.gl.DeleteShader(self.id);
		}
	}
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
	let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
	buffer.extend([b' '].iter().cycle().take(len));
	unsafe { CString::from_vec_unchecked(buffer) }
}

fn shader_from_source(
	gl: &gl::Gl, // reference to gl
	source: &CStr,
	kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, String> {
	let id = unsafe { gl.CreateShader(kind) };
	unsafe {
		gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
		gl.CompileShader(id);
	}

	let mut success: gl::types::GLint = 1;
	unsafe {
		gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
	}

	if success == 0 {
		// continue here
		let mut len: gl::types::GLint = 0;
		unsafe {
			gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
		}
		let error = create_whitespace_cstring_with_len(len as usize);
		unsafe {
			gl.GetShaderInfoLog(
				id,
				len,
				std::ptr::null_mut(),
				error.as_ptr() as *mut gl::types::GLchar,
			);
		}
		return Err(error.to_string_lossy().into_owned());
	}
	Ok(id)
}

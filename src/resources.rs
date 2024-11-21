use std::fs;
use std::io::{self, Read};
use std::ffi;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceError {
	#[error("I/O error")]
	Io(#[from] io::Error),
	#[error("Invalid file")]
	FileContainsNil,
	#[error("Unable to determine path")]
	FailedToGetExePath,
}

pub struct Resources {
	root_path: PathBuf,
}

impl Resources {
	pub fn from_relative_exe_path(rel_path: &Path) -> Result<Resources, ResourceError> {
		let exe_file_name = ::std::env::current_exe()
			.map_err(|_| ResourceError::FailedToGetExePath)?;
		let exe_path = exe_file_name.parent()
			.ok_or(ResourceError::FailedToGetExePath)?;
		Ok(Resources {
			root_path: exe_path.join(rel_path)
		})
	}

	pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, ResourceError> {
		let mut file = fs::File::open(
			resource_name_to_path(&self.root_path, resource_name)
		)?;

		let mut buffer: Vec<u8> = Vec::with_capacity(
			file.metadata()?.len() as usize + 1
		);
		file.read_to_end(&mut buffer)?;

		// check for null byte
		if buffer.iter().any(|i| *i == 0) {
			return Err(ResourceError::FileContainsNil);
		}
		Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
	}
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
	let mut path: PathBuf = root_dir.into();
	for part in location.split("/") {
		path = path.join(part);
	}
	path
}

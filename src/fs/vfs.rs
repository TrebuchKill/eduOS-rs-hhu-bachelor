// Copyright (c) 2019 Stefan Lankes, RWTH Aachen University
//
// MIT License
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Implements a simple virtual file system

use logging::*;
use errno::*;
use fs::{NodeKind, VfsNode, VfsNodeFile, VfsNodeDirectory, Vfs,
		OpenOptions, FileHandle, SeekFrom, check_path};
use fs::initrd::{RomHandle,RamHandle};
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::any::Any;
use core::fmt;
use synch::spinlock::*;

#[derive(Debug)]
struct VfsDirectory {
	/// in principle, a map with all entries of the current directory
	children: BTreeMap<String, Box<Any + core::marker::Send + core::marker::Sync>>
}

impl VfsDirectory {
	pub fn new() -> Self {
		VfsDirectory {
			children: BTreeMap::new()
		}
	}

	fn get_mut<T: VfsNode + Any>(&mut self, name: &String) -> Option<&mut T> {
		if let Some(b) = self.children.get_mut(name) {
			return b.downcast_mut::<T>();
		}
		None
	}

	fn get<T: VfsNode + Any>(&mut self, name: &String) -> Option<&T> {
		if let Some(b) = self.children.get_mut(name) {
			return b.downcast_ref::<T>();
		}
		None
	}
}

impl VfsNode for VfsDirectory {
	/// Returns the node type
	fn get_kind(&self) -> NodeKind {
		NodeKind::Directory
	}
}

impl VfsNodeDirectory for VfsDirectory {
	fn traverse_mkdir(&mut self, components: &mut Vec<&str>) -> Result<()> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			{
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					return directory.traverse_mkdir(components);
				}
			}

			let mut directory = Box::new(VfsDirectory::new());
			let result = directory.traverse_mkdir(components);
			self.children.insert(node_name, directory);

			result
		} else {
			Ok(())
		}
	}

	fn traverse_lsdir(&self, mut tabs: String) -> Result<()> {
		tabs.push_str("  ");
		for (name, node) in self.children.iter() {
			if let Some(directory) = node.downcast_ref::<VfsDirectory>() {
				info!("{}{} ({:?})", tabs, name, self.get_kind());
				directory.traverse_lsdir(tabs.clone())?;
			} else if let Some(file) = node.downcast_ref::<VfsFile>() {
				info!("{}{} ({:?})", tabs, name, file.get_kind());
			} else {
				info!("{}{} (Unknown))", tabs, name);
			}
		}

		Ok(())
	}

	fn traverse_open(&mut self, components: &mut Vec<&str>, flags: OpenOptions) -> Result<Box<FileHandle>> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				// reach endpoint => reach file
				if let Some(file) = self.get_mut::<VfsFile>(&node_name) {
					return file.get_handle(flags);
				}
			}

			if components.is_empty() == true {
				if flags.contains(OpenOptions::CREATE) {
					// Create file on demand
					let file = Box::new(VfsFile::new());
					let result = file.get_handle(flags);
					self.children.insert(node_name, file);

					result
				} else {
					Err(Error::InvalidArgument)
				}
			} else {
				// traverse to the directories to the endpoint
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					directory.traverse_open(components, flags)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}

	fn traverse_mount(&mut self, components: &mut Vec<&str>, addr: u64, len: u64) -> Result<()> {
		if let Some(component) = components.pop() {
			let node_name = String::from(component);

			if components.is_empty() == true {
				// Create file on demand
				let file = Box::new(VfsFile::new_from_rom(addr, len));
				self.children.insert(node_name, file);

				Ok(())
			} else {
				// traverse to the directories to the endpoint
				if let Some(directory) = self.get_mut::<VfsDirectory>(&node_name) {
					directory.traverse_mount(components, addr, len)
				} else {
					Err(Error::InvalidArgument)
				}
			}
		} else {
			Err(Error::InvalidArgument)
		}
	}
}


/// Enumeration of possible methods to seek within an I/O object.
#[derive(Debug,Clone)]
enum DataHandle {
	RAM(RamHandle),
	ROM(RomHandle)
}

#[derive(Debug,Clone)]
struct VfsFile {
	/// File content
	data: DataHandle
}

impl VfsFile {
	pub fn new() -> Self {
		VfsFile {
			data: DataHandle::RAM(RamHandle::new(true))
		}
	}

	pub fn new_from_rom(addr: u64, len: u64) -> Self {
		VfsFile {
			data: DataHandle::ROM(RomHandle::new(addr as *const u8, len as usize))
		}
	}
}

impl VfsNode for VfsFile {
	fn get_kind(&self) -> NodeKind {
		NodeKind::File
	}
}

impl VfsNodeFile for VfsFile {
	fn get_handle(&self, opt: OpenOptions) -> Result<Box<FileHandle>> {
		match self.data {
			DataHandle::RAM(ref data) => { Ok(Box::new(VfsFile
				{
					data: DataHandle::RAM(data.get_handle(opt))
				}))
			},
			DataHandle::ROM(ref data) => { Ok(Box::new(VfsFile
				{
					data: DataHandle::ROM(data.get_handle(opt))
				}))
			}
		}
	}
}

impl fmt::Write for VfsFile {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		match self.data {
			DataHandle::RAM(ref mut data) => { data.write_str(s) },
			_ => Err(core::fmt::Error)
		}
	}
}

impl FileHandle for VfsFile {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		match self.data {
			DataHandle::RAM(ref mut data) => { data.read(buf) },
			DataHandle::ROM(ref mut data) => { data.read(buf) }
		}
	}

	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		match self.data {
			DataHandle::RAM(ref mut data) => { data.write(buf) },
			_ => Err(Error::BadFsOperation)
		}
	}

	fn seek(&mut self, style: SeekFrom) -> Result<u64> {
		match self.data {
			DataHandle::RAM(ref mut data) => { data.seek(style) },
			DataHandle::ROM(ref mut data) => { data.seek(style) }
		}
	}
}

/// Entrypoint of the in-memory file system
#[derive(Debug)]
pub struct Fs {
	handle: Spinlock<VfsDirectory>,
}

impl Fs {
	pub fn new() -> Fs {
		Fs {
			handle: Spinlock::new(VfsDirectory::new())
		}
	}
}

impl Vfs for Fs {
	fn mkdir(&mut self, path: &String) -> Result<()> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_mkdir(&mut components)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	fn lsdir(&self) -> Result<()> {
		info!("/");

		self.handle.lock().traverse_lsdir(String::from(""))
	}

	fn open(&mut self, path: &String, flags: OpenOptions) -> Result<Box<FileHandle>> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_open(&mut components, flags)
		} else {
			Err(Error::InvalidFsPath)
		}
	}

	/// Mound memory region as file
	fn mount(&mut self, path: &String, addr: u64, len: u64) -> Result<()> {
		if check_path(path) {
			let mut components: Vec<&str> = path.split("/").collect();

			components.reverse();
			components.pop();

			self.handle.lock().traverse_mount(&mut components, addr, len)
		} else {
			Err(Error::InvalidFsPath)
		}
	}
}

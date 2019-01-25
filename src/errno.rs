// Copyright (c) 2017-2018 Stefan Lankes, RWTH Aachen University
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
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE

//! Basic error handling

use core::{result, fmt};

pub type Result<T> = result::Result<T, Error>;

/// Possible errors of eduOS-rs
#[derive(Debug,Clone)]
pub enum Error {
	/// Usage of a invalid priority
	BadPriority,
	BadFsKind,
	BadFsOperation,
	BadFsPermission,
	InvalidFsPath,
	InvalidArgument,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::BadPriority => write!(f, "Invalid priority number"),
			Error::BadFsKind => write!(f, "Bad file system kind"),
			Error::BadFsOperation => write!(f, "Bad file system operation"),
			Error::BadFsPermission => write!(f, "Bad file permission"),
			Error::InvalidFsPath => write!(f, "Invalid file system path"),
			Error::InvalidArgument => write!(f, "Inavlid argument")
		}
	}
}
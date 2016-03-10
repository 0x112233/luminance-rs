use core::marker::PhantomData;
use core::mem;
use std::vec::Vec;
use std::ops::Index;

/// Implement this trait to provide buffers.
pub trait HasBuffer {
  /// A type representing minimal information to operate on a buffer. For instance, a size, a
  /// pointer, a method to retrieve data, a handle, whatever.
  type ABuffer;

  /// Create a new buffer with a given size.
  fn new(size: usize) -> Self::ABuffer;
  /// Write values into the buffer.
  fn write_whole<T>(buffer: &Self::ABuffer, values: &Vec<T>);
  /// Write a single value in the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `Err(BufferError::Overflow)` if you provide an offset that doesn’t lie in the GPU allocated
  /// region.
  fn write<T>(buffer: &Self::ABuffer, x: T, offset: usize) -> Result<(), BufferError>;
  /// Read all values from the buffer.
  fn read_whole<T>(buffer: &Self::ABuffer) -> Vec<T>;
  /// Read a single value from the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `None` if you provide an offset that doesn’t lie in the GPU allocated region.
  fn read<T>(buffer: &Self::ABuffer, offset: usize) -> Option<&T>;
}

/// Buffer errors.
#[derive(Debug)]
pub enum BufferError {
    Overflow
  , TooManyValues
}

/// A `Buffer` is a GPU region you can picture as an array. It has a static size and cannot be
/// resized. The size is expressed in number of elements lying in the buffer, not in bytes.
#[derive(Debug)]
pub struct Buffer<C: HasBuffer, A, T> {
    repr: C::ABuffer
  , size: usize // FIXME: should be compile-time, not runtime
  , _a: PhantomData<A>
  , _t: PhantomData<T>
}

impl<C: HasBuffer, A, T> Buffer<C, A, T> {
  pub fn new(_: A, size: u32) -> Buffer<C, A, T> {
    let size = size as usize;
    let buffer = C::new(size * mem::size_of::<T>());
    Buffer { repr: buffer, size: size, _a: PhantomData, _t: PhantomData }
  }

  pub fn get(&self, i: u32) -> Option<&T> {
    C::read(&self.repr, i as usize * mem::size_of::<T>())
  }
}

impl<C: HasBuffer, A, T> Buffer<C, A, T> where T: Clone {
  /// Fill a `Buffer` with a single value.
  pub fn clear(&self, x: T) {
    C::write_whole(&self.repr, &vec![x; self.size]);
  }
}

impl<C: HasBuffer, A, T> Index<u32> for Buffer<C, A, T> {
  type Output = T;

  fn index(&self, i: u32) -> &T {
		self.get(i).unwrap()
  }
}
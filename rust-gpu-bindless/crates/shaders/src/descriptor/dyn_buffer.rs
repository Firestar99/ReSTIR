use crate::buffer_content::{BufferContent, BufferStruct, Metadata, MetadataCpuInterface};
use crate::descriptor::{Buffer, Desc, DescRef};
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use rust_gpu_bindless_macros::BufferStruct;

/// A dynamic Buffer whose contents are unidentified. Use [`DynBuffer::can_upcast`] to check whether it is of some type
/// and [`DynBuffer::upcast`] to upcast it to said type.
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, BufferStruct)]
pub struct DynBuffer<R: DescRef> {
	id: DynBufferType,
	desc: Desc<R, Buffer<[u8]>>,
}

impl<R: DescRef> DynBuffer<R> {
	/// Create a new [`DynBuffer`]
	pub fn new<T: BufferContent + ?Sized>(type_id: BufferType<T>, desc: Desc<R, Buffer<T>>) -> Self {
		unsafe {
			Self {
				id: type_id.to_untyped(),
				desc: desc.transmute_buffer::<[u8]>(),
			}
		}
	}

	/// Create a new [`DynBuffer`] that cannot be [`Self::upcast`] back into it's concrete type
	pub fn new_undefined<T: BufferContent + ?Sized>(desc: Desc<R, Buffer<T>>) -> Self {
		unsafe {
			Self {
				id: DynBufferType::UNDEFINED,
				desc: desc.transmute_buffer::<[u8]>(),
			}
		}
	}

	/// Check if this [`DynBuffer`] can be [`Self::upcast`] to a specific [`BufferType`] (or [`DynBufferType`])
	pub fn can_upcast(&self, type_id: impl AsRef<DynBufferType>) -> bool {
		self.id == *type_id.as_ref()
	}

	// How I'd love to return Option right here...
	/// Upcast this buffer to a concrete buffer type using its respective [`BufferType`]. Will panic if the
	/// [`BufferType`] does not match the expected value, so check beforehand with [`Self::can_upcast`].
	pub fn upcast<T: BufferContent + ?Sized>(self, type_id: BufferType<T>) -> Desc<R, Buffer<T>> {
		if self.can_upcast(type_id) {
			unsafe { self.upcast_unchecked::<T>(type_id) }
		} else {
			panic!(
				"DynMaterialType {:?} is different from attempted upcast target {:?}",
				self.id.to_u32(),
				type_id.to_untyped().to_u32()
			)
		}
	}

	/// Upcast this buffer to a concrete buffer type, unchecked
	///
	/// # Safety
	/// Does not check if the buffer types match
	pub unsafe fn upcast_unchecked<T: BufferContent + ?Sized>(self, _: BufferType<T>) -> Desc<R, Buffer<T>> {
		unsafe { self.desc.transmute_buffer::<T>() }
	}

	/// Destructure into raw parts, like a slice
	///
	/// # Safety
	/// Do not access the buffer, the type is incorrect
	pub unsafe fn to_raw_parts(&self) -> (&DynBufferType, &Desc<R, Buffer<[u8]>>) {
		(&self.id, &self.desc)
	}

	/// Destructure into raw parts, like a slice
	///
	/// # Safety
	/// Do not access the buffer, the type is incorrect
	pub unsafe fn into_raw_parts(self) -> (DynBufferType, Desc<R, Buffer<[u8]>>) {
		(self.id, self.desc)
	}

	/// Construct from raw parts, like a slice
	///
	/// # Safety
	/// Values must stem from [`Self::into_raw_parts`], at most the descriptor's [`DescRef`] is allowed to change
	pub unsafe fn from_raw_parts(id: DynBufferType, desc: Desc<R, Buffer<[u8]>>) -> Self {
		Self { id, desc }
	}
}

/// A dynamic ID without generics that could represent any type.
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, BufferStruct)]
pub struct DynBufferType(u32);

impl DynBufferType {
	pub const UNDEFINED: DynBufferType = unsafe { BufferType::<()>::new_unchecked(0).to_untyped() };

	pub fn to_u32(&self) -> u32 {
		self.0
	}
}

impl AsRef<DynBufferType> for DynBufferType {
	fn as_ref(&self) -> &DynBufferType {
		self
	}
}

/// A concrete type of some T with it's associated [`DynBufferType`]
#[repr(C)]
pub struct BufferType<T: BufferContent + ?Sized> {
	id: DynBufferType,
	_phantom: PhantomData<T>,
}

impl<T: BufferContent + ?Sized> BufferType<T> {
	/// Create a new [`BufferType`] from an u32 ID
	///
	/// # Safety
	/// The ID must be uniquely assigned to the type T, there must be no other instance with the same ID but a different
	/// T generic. This is to ensure [`DynBuffer::upcast`] doesn't transmute data into the wrong type.
	pub const unsafe fn new_unchecked(id: u32) -> Self {
		Self {
			id: DynBufferType(id),
			_phantom: PhantomData {},
		}
	}

	pub const fn to_untyped(&self) -> DynBufferType {
		self.id
	}

	pub fn to_u32(&self) -> u32 {
		self.id.0
	}
}

impl<T: BufferContent + ?Sized> AsRef<DynBufferType> for BufferType<T> {
	fn as_ref(&self) -> &DynBufferType {
		&self.id
	}
}

impl<T: BufferContent + ?Sized> Clone for BufferType<T> {
	fn clone(&self) -> Self {
		Self {
			id: self.id,
			_phantom: self._phantom,
		}
	}
}

impl<T: BufferContent + ?Sized> Copy for BufferType<T> {}

impl<T: BufferContent + ?Sized> PartialEq<Self> for BufferType<T> {
	fn eq(&self, other: &Self) -> bool {
		self.id.eq(&other.id)
	}
}

impl<T: BufferContent + ?Sized> Eq for BufferType<T> {}

impl<T: BufferContent + ?Sized> Debug for BufferType<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("DynMaterialId").field(&self.id).finish()
	}
}

// This should be `BufferStructPlain`, but it additionally requires `Send + Sync + 'static`, even though this type
// doesn't need to be any of it
unsafe impl<T: BufferContent + ?Sized> BufferStruct for BufferType<T> {
	type Transfer = u32;

	unsafe fn write_cpu(self, _: &mut impl MetadataCpuInterface) -> Self::Transfer {
		self.id.0
	}

	unsafe fn read(from: Self::Transfer, _: Metadata) -> Self {
		unsafe { Self::new_unchecked(from) }
	}
}

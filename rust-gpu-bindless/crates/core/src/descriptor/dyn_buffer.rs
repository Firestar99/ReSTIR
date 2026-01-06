use crate::descriptor::{RC, RCDescExt};
use crate::platform::BindlessPlatform;
use rust_gpu_bindless_shaders::buffer_content::BufferContent;
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::{BufferType, DynBuffer};
use rust_gpu_bindless_shaders::descriptor::{Strong, Transient, TransientAccess};
use std::sync::atomic::{AtomicU32, Ordering};

static DYN_REF: AtomicU32 = AtomicU32::new(1);

/// Register a new dynamic [`BufferType`]
///
/// You don't need to register every buffer type, only if you need [`BufferType`] to create new [`DynBuffer`]s or
/// [`DynBuffer::upcast`] them. You can also use [`DynBuffer::new_undefined`] to create a [`DynBuffer`] of any type
/// without allocating a [`BufferType`], though you can't upcast them back to a concrete buffer.
///
/// [`DynBuffer`]: rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer
/// [`DynBuffer::upcast`]: rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer::upcast
/// [`DynBuffer::new_undefined`]: rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer::new_undefined
pub fn register_dyn_buffer_type<T: BufferContent + ?Sized>() -> BufferType<T> {
	let id = DYN_REF.fetch_add(1, Ordering::Relaxed);
	if id == u32::MAX {
		panic!("`dyn_buffer::DYN_REF` overflowed!")
	} else {
		// Safety: Atomic ensures this is unique
		unsafe { BufferType::new_unchecked(id) }
	}
}

pub trait DynBufferRCExt {
	fn to_strong(&self) -> DynBuffer<Strong>;
	fn to_transient<'a>(&self, access: &impl TransientAccess<'a>) -> DynBuffer<Transient<'a>>;
}

impl<P: BindlessPlatform> DynBufferRCExt for DynBuffer<RC<P>> {
	fn to_strong(&self) -> DynBuffer<Strong> {
		unsafe {
			let (id, desc) = self.to_raw_parts();
			DynBuffer::from_raw_parts(*id, desc.to_strong())
		}
	}

	fn to_transient<'a>(&self, access: &impl TransientAccess<'a>) -> DynBuffer<Transient<'a>> {
		unsafe {
			let (id, desc) = self.to_raw_parts();
			DynBuffer::from_raw_parts(*id, desc.to_transient(access))
		}
	}
}

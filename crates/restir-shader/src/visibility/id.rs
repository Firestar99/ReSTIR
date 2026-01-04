use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use rust_gpu_bindless_macros::BufferStructPlain;
use static_assertions::const_assert_eq;

pub const TRIANGLE_BITS: u32 = 20;
pub const INSTANCE_BITS: u32 = 12;

const TRIANGLE_MASK: u32 = (1 << TRIANGLE_BITS) - 1;
const INSTANCE_MASK: u32 = (1 << INSTANCE_BITS) - 1;

const TRIANGLE_SHIFT: u32 = 0;
const INSTANCE_SHIFT: u32 = TRIANGLE_SHIFT + TRIANGLE_BITS;

// uses all 32 TRIANGLE_BITS
const_assert_eq!(TRIANGLE_BITS + INSTANCE_BITS, 32);
// masks use entire 32 bit range
const_assert_eq!(TRIANGLE_MASK << TRIANGLE_SHIFT | INSTANCE_MASK << INSTANCE_SHIFT, !0);
// masks do not overlap
const_assert_eq!(TRIANGLE_MASK << TRIANGLE_SHIFT & INSTANCE_MASK << INSTANCE_SHIFT, 0);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, BufferStructPlain)]
pub struct TriangleId(u32);
const_assert_eq!(size_of::<TriangleId>(), 4);

#[derive(Clone, Debug)]
pub struct TriangleIdOutOfRange;

impl Error for TriangleIdOutOfRange {}

impl Display for TriangleIdOutOfRange {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(
			f,
			"Triangle id can't fit into {TRIANGLE_BITS} bits (max {TRIANGLE_MASK})"
		)
	}
}

impl TriangleId {
	/// Creates a new `TriangleId`. Returns `None` if the version is too large to be represented by [`TRIANGLE_BITS`]
	/// bits.
	pub const fn new(id: u32) -> Result<Self, TriangleIdOutOfRange> {
		if id == id & TRIANGLE_MASK {
			unsafe { Ok(Self::new_unchecked(id)) }
		} else {
			Err(TriangleIdOutOfRange)
		}
	}

	/// # Safety
	/// See [`Self::new`]. The supplied `type_id` must fit into [`ID_TYPE_BITS`] bits.
	pub const unsafe fn new_unchecked(type_id: u32) -> Self {
		Self(type_id)
	}

	pub const fn to_u32(&self) -> u32 {
		self.0
	}

	pub const fn to_usize(&self) -> usize {
		self.0 as usize
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, BufferStructPlain)]
pub struct InstanceId(u32);
const_assert_eq!(size_of::<InstanceId>(), 4);

#[derive(Clone, Debug)]
pub struct InstanceIdOutOfRange;

impl Error for InstanceIdOutOfRange {}

impl Display for InstanceIdOutOfRange {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(
			f,
			"Instance id can't fit into {INSTANCE_BITS} bits (max {INSTANCE_MASK})"
		)
	}
}

impl InstanceId {
	/// Creates a new `Instance`. Returns `None` if the version is too large to be represented by [`INSTANCE_BITS`]
	/// bits.
	pub const fn new(id: u32) -> Result<Self, InstanceIdOutOfRange> {
		if id == id & INSTANCE_MASK {
			unsafe { Ok(Self::new_unchecked(id)) }
		} else {
			Err(InstanceIdOutOfRange)
		}
	}

	/// # Safety
	/// See [`Self::new`]. The supplied `type_id` must fit into [`ID_TYPE_BITS`] bits.
	pub const unsafe fn new_unchecked(type_id: u32) -> Self {
		Self(type_id)
	}

	pub const fn to_u32(&self) -> u32 {
		self.0
	}

	pub const fn to_usize(&self) -> usize {
		self.0 as usize
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct GeometryId {
	pub instance_id: InstanceId,
	pub triangle_id: TriangleId,
	pub is_clear: bool,
}

impl GeometryId {
	pub const fn pack(&self) -> PackedGeometryId {
		PackedGeometryId::new(self.instance_id, self.triangle_id)
	}
}

#[repr(transparent)]
#[derive(Copy, Clone, Hash, Eq, PartialEq, BufferStructPlain)]
pub struct PackedGeometryId(u32);
const_assert_eq!(size_of::<PackedGeometryId>(), 4);

impl PackedGeometryId {
	pub const CLEAR: Self = Self(!0);

	pub const fn new(instance_id: InstanceId, triangle_id: TriangleId) -> Self {
		let mut value = 0;
		value |= (instance_id.0 & INSTANCE_MASK) << INSTANCE_SHIFT;
		value |= (triangle_id.0 & TRIANGLE_MASK) << TRIANGLE_SHIFT;
		Self(value)
	}

	pub const fn unpack(&self) -> GeometryId {
		GeometryId {
			instance_id: InstanceId((self.0 >> INSTANCE_SHIFT) & INSTANCE_MASK),
			triangle_id: TriangleId((self.0 >> TRIANGLE_SHIFT) & TRIANGLE_MASK),
			is_clear: self.is_clear(),
		}
	}

	pub const fn is_clear(&self) -> bool {
		self.0 == Self::CLEAR.0
	}

	pub const fn from_u32(value: u32) -> Self {
		Self(value)
	}

	pub const fn to_u32(&self) -> u32 {
		self.0
	}
}

impl Debug for PackedGeometryId {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("PackedGeometryId").field(&self.unpack()).finish()
	}
}

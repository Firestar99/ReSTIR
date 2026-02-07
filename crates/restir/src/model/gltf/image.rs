use ash::vk::{BufferImageCopy2, CopyBufferToImageInfo2, Extent3D, ImageAspectFlags, ImageSubresourceLayers, Offset3D};
use glam::{UVec2, Vec4};
use image::Rgba;
use rust_gpu_bindless::descriptor::{
	Bindless, BindlessAllocationScheme, BindlessBufferCreateInfo, BindlessBufferUsage, BindlessImageCreateInfo,
	BindlessImageUsage, Extent, Format, RCDesc,
};
use rust_gpu_bindless::pipeline::{
	HasResourceContext, ImageAccessType, MutBufferAccessExt, MutImageAccessExt, TransferRead, TransferWrite,
};
use rust_gpu_bindless_shaders::descriptor::{Image, Image2d};
use std::future::Future;

pub fn single_pixel_image(color: Vec4) -> image::RgbaImage {
	image::RgbaImage::from_fn(1, 1, |_, _| Rgba(color.to_array().map(|c| (c * 255.) as u8)))
}

pub fn upload_image<'a>(
	bindless: &'a Bindless,
	image: &image::RgbaImage,
	name: &str,
) -> impl Future<Output = anyhow::Result<RCDesc<Image<Image2d>>>> + use<'a> {
	let result: anyhow::Result<_> = (|| {
		let extent = Extent::from(UVec2::from(image.dimensions()));

		let staging_buffer = {
			profiling::scope!("image upload to host buffer");
			let upload_buffer = bindless.buffer().alloc_from_iter(
				&BindlessBufferCreateInfo {
					usage: BindlessBufferUsage::MAP_WRITE | BindlessBufferUsage::TRANSFER_SRC,
					name: &format!("staging buffer: {name}"),
					allocation_scheme: BindlessAllocationScheme::AllocatorManaged,
				},
				image.as_raw().iter().copied(),
			)?;
			upload_buffer
		};

		let gpu_image = {
			profiling::scope!("image alloc");
			bindless.image().alloc(&BindlessImageCreateInfo {
				format: Format::R8G8B8A8_SRGB,
				extent,
				mip_levels: 1,
				usage: BindlessImageUsage::SAMPLED | BindlessImageUsage::TRANSFER_DST,
				name,
				..BindlessImageCreateInfo::default()
			})?
		};

		{
			profiling::scope!("image copy cmd");
			Ok(bindless.execute(|cmd| {
				let buffer = staging_buffer.access::<TransferRead>(cmd)?;
				let gpu_image = gpu_image.access::<TransferWrite>(cmd)?;

				unsafe {
					cmd.ash_flush();
					let device = &cmd.bindless().platform.device;
					let buffer = buffer.inner_slot();
					let gpu_image = gpu_image.inner_slot();
					let region = BufferImageCopy2 {
						buffer_offset: 0,
						buffer_row_length: 0,
						buffer_image_height: 0,
						image_subresource: ImageSubresourceLayers {
							aspect_mask: ImageAspectFlags::COLOR,
							mip_level: 0,
							base_array_layer: 0,
							layer_count: gpu_image.array_layers,
						},
						image_offset: Offset3D::default(),
						image_extent: Extent3D::from(extent),
						..Default::default()
					};

					device.cmd_copy_buffer_to_image2(
						cmd.ash_command_buffer(),
						&CopyBufferToImageInfo2::default()
							.src_buffer(buffer.buffer)
							.dst_image(gpu_image.image)
							.dst_image_layout(TransferWrite::IMAGE_ACCESS.to_ash_image_access().image_layout)
							.regions(&[region]),
					);
				}
				Ok(gpu_image.into_shared())
			})?)
		}
	})();
	async { Ok(result?.await) }
}

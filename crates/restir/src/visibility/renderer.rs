use crate::material::debug::VisiDebugPipeline;
use crate::visibility::raster::VisiRasterPipeline;
use crate::visibility::scene::VisiCpuScene;
use anyhow::anyhow;
use glam::UVec4;
use restir_shader::material::debug::DebugSettings;
use rust_gpu_bindless::descriptor::{
	Bindless, BindlessAllocationScheme, BindlessImageCreateInfo, BindlessImageUsage, Extent, Format, Image2d, Image2dU,
	ImageDescExt, MutDesc, MutImage, RCDescExt,
};
use rust_gpu_bindless::pipeline::{
	ColorAttachment, DepthStencilAttachment, ImageAccessType, LoadOp, MutImageAccess, MutImageAccessExt, Recording,
	RenderPassFormat, RenderingAttachment, RenderingAttachmentImage, SampledRead, StorageReadWrite, StoreOp,
};
use smallvec::SmallVec;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct VisiPipelinesFormat {
	pub depth: Format,
	pub visi: Format,
	pub debug_output_format: Format,
}

impl VisiPipelinesFormat {
	pub fn new(_bindless: &Bindless, debug_output_format: Format) -> Self {
		Self {
			depth: Format::D32_SFLOAT,
			visi: Format::R32_UINT,
			debug_output_format,
		}
	}

	pub fn to_render_pass_format(&self) -> RenderPassFormat {
		RenderPassFormat {
			color_attachments: SmallVec::from_slice(&[self.visi]),
			depth_attachment: Some(self.depth),
		}
	}
}

pub struct VisiPipelines {
	bindless: Bindless,
	format: VisiPipelinesFormat,
	raster_pipeline: VisiRasterPipeline,
	debug_pipeline: VisiDebugPipeline,
}

impl VisiPipelines {
	pub fn new(bindless: &Bindless, format: VisiPipelinesFormat) -> anyhow::Result<Arc<Self>> {
		Ok(Arc::new(Self {
			bindless: bindless.clone(),
			format,
			raster_pipeline: VisiRasterPipeline::new(&bindless, format)?,
			debug_pipeline: VisiDebugPipeline::new(&bindless)?,
		}))
	}

	pub fn new_renderer(self: &Arc<Self>) -> VisiRenderer {
		VisiRenderer::new(self.clone())
	}
}

pub struct VisiRenderer {
	pub pipeline: Arc<VisiPipelines>,
	resources: Option<VisiRendererResources>,
}

pub struct VisiRendererResources {
	pub extent: Extent,
	pub packed_vertex_image: MutDesc<MutImage<Image2dU>>,
	pub depth: MutDesc<MutImage<Image2d>>,
}

impl VisiRendererResources {
	pub fn new(renderer: &VisiPipelines, extent: Extent) -> anyhow::Result<Self> {
		let packed_vertex_image = renderer.bindless.image().alloc(&BindlessImageCreateInfo {
			format: renderer.format.visi,
			extent,
			mip_levels: 1,
			array_layers: 1,
			samples: Default::default(),
			usage: BindlessImageUsage::COLOR_ATTACHMENT | BindlessImageUsage::SAMPLED,
			allocation_scheme: BindlessAllocationScheme::Dedicated,
			name: "packed_vertex_image",
			..BindlessImageCreateInfo::default()
		})?;
		let depth = renderer.bindless.image().alloc(&BindlessImageCreateInfo {
			format: renderer.format.depth,
			extent,
			mip_levels: 1,
			array_layers: 1,
			samples: Default::default(),
			usage: BindlessImageUsage::DEPTH_STENCIL_ATTACHMENT,
			allocation_scheme: BindlessAllocationScheme::Dedicated,
			name: "depth",
			..BindlessImageCreateInfo::default()
		})?;

		Ok(Self {
			extent,
			packed_vertex_image,
			depth,
		})
	}
}

pub struct VisiRenderInfo {
	pub scene: VisiCpuScene,
	pub debug_settings: DebugSettings,
}

impl VisiRenderer {
	pub fn new(pipeline: Arc<VisiPipelines>) -> Self {
		Self {
			pipeline,
			resources: None,
		}
	}

	pub fn render(
		&mut self,
		cmd: &mut Recording<'_>,
		output_image: &MutImageAccess<'_, Image2d, StorageReadWrite>,
		info: VisiRenderInfo,
	) -> anyhow::Result<()> {
		self.image_supported(output_image)?;
		let resources = {
			let extent = output_image.extent();
			let resources = if let Some(resources) = self.resources.take() {
				if resources.extent == extent {
					Some(resources)
				} else {
					drop(resources);
					None
				}
			} else {
				None
			};
			if let Some(resources) = resources {
				resources
			} else {
				VisiRendererResources::new(&self.pipeline, extent)?
			}
		};

		let mut packed_vertex_image = resources.packed_vertex_image.access_dont_care::<ColorAttachment>(cmd)?;
		let mut depth = resources.depth.access::<DepthStencilAttachment>(cmd)?;
		cmd.begin_rendering(
			self.pipeline.format.to_render_pass_format(),
			&[RenderingAttachment {
				image: RenderingAttachmentImage::ColorU {
					image: &mut packed_vertex_image,
					clear_value: UVec4::splat(!0),
				},
				load_op: LoadOp::Clear,
				store_op: StoreOp::Store,
			}],
			Some(RenderingAttachment {
				image: RenderingAttachmentImage::DepthStencil {
					image: &mut depth,
					clear_depth: 1.0,
					clear_stencil: 0,
				},
				load_op: LoadOp::Clear,
				store_op: StoreOp::DontCare,
			}),
			|mut rp| {
				let scene_buffer = info.scene.scene.to_transient(rp);
				for draw in &info.scene.draws {
					self.pipeline.raster_pipeline.draw(&mut rp, scene_buffer, draw)?;
				}
				Ok(())
			},
		)?;

		let packed_vertex_image = packed_vertex_image.transition::<SampledRead>()?;
		self.pipeline.debug_pipeline.image.dispatch(
			cmd,
			info.scene,
			packed_vertex_image.to_transient_sampled()?,
			output_image.to_mut_transient(),
			info.debug_settings,
		)?;

		self.resources = Some(VisiRendererResources {
			extent: resources.extent,
			packed_vertex_image: packed_vertex_image.into_desc(),
			depth: depth.into_desc(),
		});
		Ok(())
	}

	pub fn image_supported(&self, output_image: &MutImageAccess<Image2d, impl ImageAccessType>) -> anyhow::Result<()> {
		let extent = output_image.extent();
		if output_image.format() != self.pipeline.format.debug_output_format {
			Err(anyhow!(
				"Expected format {:?} but output_image has format {:?}",
				self.pipeline.format.debug_output_format,
				output_image.format()
			))
		} else if extent.depth != 1 {
			Err(anyhow!("Image was not 2D"))
		} else {
			Ok(())
		}
	}
}

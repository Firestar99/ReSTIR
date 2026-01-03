use crate::visibility::renderer::VisiPipelinesFormat;
use crate::visibility::scene::VisiCpuDraw;
use ash::vk::{ColorComponentFlags, CompareOp, PipelineColorBlendAttachmentState, PrimitiveTopology};
use restir_shader::visibility::raster::Param;
use restir_shader::visibility::scene::VisiScene;
use rust_gpu_bindless::descriptor::{Bindless, Buffer, RCDescExt, TransientDesc};
use rust_gpu_bindless::pipeline::{
	BindlessGraphicsPipeline, DrawIndexedIndirectCommand, GraphicsPipelineCreateInfo,
	PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineInputAssemblyStateCreateInfo,
	PipelineRasterizationStateCreateInfo, RecordingError, Rendering,
};

pub struct VisiRasterPipeline {
	pipeline: BindlessGraphicsPipeline<Param<'static>>,
}

impl VisiRasterPipeline {
	pub fn new(bindless: &Bindless, format: VisiPipelinesFormat) -> anyhow::Result<Self> {
		Ok(Self {
			pipeline: bindless.create_graphics_pipeline(
				&format.to_render_pass_format(),
				&GraphicsPipelineCreateInfo {
					input_assembly_state: PipelineInputAssemblyStateCreateInfo::default()
						.topology(PrimitiveTopology::TRIANGLE_LIST),
					rasterization_state: PipelineRasterizationStateCreateInfo::default().line_width(1.0),
					depth_stencil_state: PipelineDepthStencilStateCreateInfo::default()
						.depth_test_enable(true)
						.depth_write_enable(true)
						.depth_compare_op(CompareOp::LESS),
					color_blend_state: PipelineColorBlendStateCreateInfo::default().attachments(&[
						PipelineColorBlendAttachmentState::default().color_write_mask(ColorComponentFlags::RGBA),
					]),
				},
				crate::shader::visibility::raster::visibility_vert::new(),
				crate::shader::visibility::raster::visibility_frag::new(),
			)?,
		})
	}

	pub fn draw(
		&self,
		rp: &mut Rendering,
		scene: TransientDesc<Buffer<VisiScene>>,
		draw: &VisiCpuDraw,
	) -> Result<(), RecordingError> {
		rp.draw_indexed(
			&self.pipeline,
			&draw.model.indices,
			DrawIndexedIndirectCommand {
				index_count: draw.model.indices_count,
				instance_count: draw.instance_count,
				first_index: 0,
				vertex_offset: 0,
				first_instance: draw.instance_start,
			},
			Param {
				scene,
				model: draw.model.model.to_transient(rp),
			},
		)?;
		Ok(())
	}
}

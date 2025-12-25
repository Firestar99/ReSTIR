use rust_gpu_bindless_shader_builder::ShaderSymbolsBuilder;

fn main() -> anyhow::Result<()> {
	ShaderSymbolsBuilder::new("restir-shader", "spirv-unknown-vulkan1.3")?.build()?;
	Ok(())
}

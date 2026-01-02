use crate::controls::app_focus::AppFocus;
use crate::controls::delta_time::DeltaTimer;
use crate::controls::fps_camera_controller::FpsCameraController;
use crate::debugger;
use crate::visibility::renderer::{VisiPipelines, VisiPipelinesFormat};
use crate::visibility::scene::CpuSceneAccum;
use glam::{Affine3A, UVec3, Vec3, Vec3Swizzles};
use restir_shader::camera::Camera;
use restir_shader::utils::affine_transform::AffineTransform;
use restir_shader::visibility::scene::InstanceInfo;
use rust_gpu_bindless::descriptor::{BindlessImageUsage, BindlessInstance, DescriptorCounts, ImageDescExt};
use rust_gpu_bindless::pipeline::{MutImageAccessExt, Present, StorageReadWrite};
use rust_gpu_bindless::platform::ash::Debuggers;
use rust_gpu_bindless::platform::ash::{AshSingleGraphicsQueueCreateInfo, ash_init_single_graphics_queue};
use rust_gpu_bindless_winit::ash::{
	AshSwapchain, AshSwapchainParams, SwapchainImageFormatPreference, ash_enumerate_required_extensions,
};
use rust_gpu_bindless_winit::event_loop::{EventLoopExecutor, event_loop_init};
use rust_gpu_bindless_winit::window_ref::WindowRef;
use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use winit::event::{Event, WindowEvent};
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::WindowAttributes;

pub fn main() {
	event_loop_init(|event_loop, events| async {
		main_loop(event_loop, events).await.unwrap();
	});
}

pub async fn main_loop(event_loop: EventLoopExecutor, events: Receiver<Event<()>>) -> anyhow::Result<()> {
	if matches!(debugger(), Debuggers::RenderDoc) {
		unsafe {
			// renderdoc does not yet support wayland
			std::env::remove_var("WAYLAND_DISPLAY");
			std::env::set_var("ENABLE_VULKAN_RENDERDOC_CAPTURE", "1");
		}
	}

	let (window, window_extensions) = event_loop
		.spawn(|e| {
			let window = e.create_window(WindowAttributes::default().with_title("swapchain triangle"))?;
			let extensions = ash_enumerate_required_extensions(e.display_handle()?.as_raw())?;
			Ok::<_, anyhow::Error>((WindowRef::new(Arc::new(window)), extensions))
		})
		.await?;

	let bindless = unsafe {
		BindlessInstance::new(
			ash_init_single_graphics_queue(AshSingleGraphicsQueueCreateInfo {
				instance_extensions: window_extensions,
				extensions: &[ash::khr::swapchain::NAME],
				debug: debugger(),
				..AshSingleGraphicsQueueCreateInfo::default()
			})?,
			DescriptorCounts::REASONABLE_DEFAULTS,
		)
	};

	let mut swapchain = unsafe {
		let bindless2 = bindless.clone();
		AshSwapchain::new(&bindless, &event_loop, window.clone(), move |surface, _| {
			AshSwapchainParams::automatic_best(
				&bindless2,
				surface,
				BindlessImageUsage::STORAGE,
				SwapchainImageFormatPreference::UNORM,
			)
		})
	}
	.await?;

	let visi_format = VisiPipelinesFormat::new(&bindless, swapchain.params().format);
	let visi_pipelines = VisiPipelines::new(&bindless, visi_format)?;
	let mut visi_renderer = visi_pipelines.new_renderer();

	let model = crate::visibility::debug_models::cube(&bindless, Affine3A::default())?;

	let mut delta_timer = DeltaTimer::new();
	let mut app_focus = AppFocus::new(event_loop.clone(), window);
	let mut camera_controls = FpsCameraController::default();

	'outer: loop {
		{
			profiling::scope!("event handling");
			for event in events.try_iter() {
				swapchain.handle_input(&event);
				if !app_focus.handle_input(&event) {
					camera_controls.handle_input(&event, app_focus.game_focused);
				}

				if let Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} = &event
				{
					break 'outer;
				}
			}
		}

		let swapchain_image = {
			profiling::scope!("swapchain image acquire");
			swapchain.acquire_image(None).await?
		};

		let scene;
		{
			profiling::scope!("update");
			let delta_time = delta_timer.next();

			let out_extent = UVec3::from(swapchain_image.extent()).xy();
			let fov_y = 90.;
			let camera = Camera::new_perspective_rh_y_flip(
				out_extent,
				fov_y / 360. * 2. * PI,
				0.01,
				1000.,
				AffineTransform::new(camera_controls.update(delta_time)),
			);

			let mut accum = CpuSceneAccum::new();
			accum.push(
				&model,
				InstanceInfo {
					world_from_local: AffineTransform::new(Affine3A::from_translation(Vec3::new(0., 0., -2.))),
				},
			);
			scene = accum.finish(&bindless, camera)?;
		}

		let swapchain_image = {
			profiling::scope!("render");
			bindless.execute(|mut cmd| {
				let rt = swapchain_image.access_dont_care::<StorageReadWrite>(&cmd)?;
				visi_renderer.render(&mut cmd, scene, &rt).unwrap();
				Ok(rt.transition::<Present>()?.into_desc())
			})?
		};

		{
			profiling::scope!("swapchain image present");
			swapchain.present_image(swapchain_image)?;
		}
		profiling::finish_frame!();
	}

	Ok(())
}

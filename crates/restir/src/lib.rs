use rust_gpu_bindless::platform::ash::Debuggers;

pub mod controls;
pub mod main_loop;
pub mod model;
pub mod shader;
pub mod visibility;

/// the global setting on which debugger to use for integration tests
pub fn debugger() -> Debuggers {
	// Validation layer does not yet support timelime semaphores properly, leading to many false positives.
	// On Linux RADV gpu assisted validation even segfaulting on graphics pipeline creation.
	Debuggers::Validation
}

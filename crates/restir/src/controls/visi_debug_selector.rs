use egui::{SliderClamping, Ui};
use restir_shader::material::debug::{DebugSettings, DebugType};

#[derive(Debug, Default)]
pub struct VisiDebugSettings {
	pub s: DebugSettings,
}

impl VisiDebugSettings {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self) -> DebugSettings {
		DebugSettings {
			debug_type: self.s.debug_type,
			debug_mix: if self.s.debug_type == DebugType::None {
				0.
			} else {
				self.s.debug_mix
			},
			view_range: self.s.view_range,
		}
	}

	pub fn ui(&mut self, ui: &mut Ui) {
		ui.strong("Visibility Debug View:");
		egui::ComboBox::from_id_salt(concat!(file!(), line!()))
			.selected_text(format!("{:?}", self.s.debug_type))
			.show_ui(ui, |ui| {
				for x in (0..DebugType::LEN).map(|i| DebugType::try_from(i).unwrap()) {
					ui.selectable_value(&mut self.s.debug_type, x, format!("{:?}", x));
				}
			});
		let debug_enabled = self.s.debug_type != DebugType::None;
		ui.add_enabled(
			debug_enabled,
			egui::Slider::new(&mut self.s.debug_mix, 0. ..=1.).text("color mix"),
		);
		ui.add_enabled(
			debug_enabled,
			egui::Slider::new(&mut self.s.view_range.min, 0..=128)
				.text("range min")
				.clamping(SliderClamping::Never),
		);
		ui.add_enabled(
			debug_enabled,
			egui::Slider::new(&mut self.s.view_range.max, 0..=128)
				.text("range max")
				.clamping(SliderClamping::Never),
		);
		ui.add_enabled(
			debug_enabled,
			egui::Checkbox::new(&mut self.s.view_range.wrap, "Wrap values"),
		);
	}
}

use egui::Ui;
use restir_shader::visibility::debug::{DebugSettings, DebugType};

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
		ui.add_enabled(
			self.s.debug_type != DebugType::None,
			egui::Slider::new(&mut self.s.debug_mix, 0. ..=1.).text("color mix"),
		);
	}
}

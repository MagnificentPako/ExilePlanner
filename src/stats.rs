use imgui::Ui;

pub enum StatType {
    Number(f32),
    String(String),
    Percentage(f32),
}

pub struct Stat {
    pub name: String,
    pub value: StatType,
    pub name_color: [f32; 4],
    pub value_color: [f32; 4],
}

impl std::fmt::Display for StatType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StatType::Number(n) => write!(f, "{}", n),
            StatType::String(s) => write!(f, "{:.1}", s),
            StatType::Percentage(p) => write!(f, "{:.1}%", (p * 100.0)),
        }
    }
}

impl Stat {
    pub fn show(&self, ui: &Ui) {
        let name_color = ui.push_style_color(imgui::StyleColor::Text, self.name_color);
        ui.text(&self.name);
        name_color.pop();
        ui.next_column();

        let value_color = ui.push_style_color(imgui::StyleColor::Text, self.value_color);
        ui.text(format!("{}", self.value));
        value_color.pop();
        ui.next_column();
    }
}

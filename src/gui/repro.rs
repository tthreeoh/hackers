pub struct Ui;

pub struct ImguiBackend<'ui> {
    ui: &'ui Ui,
}

pub struct SelectableBuilder<'ui, 'label> {
    pub ui: &'ui Ui,
    pub label: &'label str,
}

pub trait UiBackend {
    fn selectable<'a>(&self, label: &'a str) -> SelectableBuilder<'_, 'a>;
}

impl<'ui> UiBackend for ImguiBackend<'ui> {
    fn selectable<'a>(&self, label: &'a str) -> SelectableBuilder<'_, 'a> {
        SelectableBuilder { ui: self.ui, label }
    }
}

fn main() {
    let ui = Ui;
    let backend = ImguiBackend { ui: &ui };
    let label = "test";
    let _builder = backend.selectable(label);
}

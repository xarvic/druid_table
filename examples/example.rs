use std::sync::Arc;
use druid::{AppLauncher, ArcStr, UnitPoint, Widget, WidgetExt, WindowDesc, Data, Lens};
use druid::im::Vector;
use druid::lens::Identity;
use druid::widget::{Axis, Button, Flex, Slider, TextBox};
use druid_table::Table;

#[derive(Clone, Data, Lens)]
struct AppData {
    name: Arc<String>,
    count: f64,
}

fn root_widget() -> impl Widget<Vector<AppData>> {
    let table = Table::new_static(Axis::Vertical)
        .with_line(Identity, AppData::name, ||TextBox::multiline())
        .with_line(Identity, AppData::count, ||Slider::new().with_range(0.0, 10.0));

    Flex::column()
        .with_child(table)
        .with_child(
            Button::new("Add")
                .on_click(|_, data: &mut Vector<AppData>, _|data.push_back(AppData {
                    name: Arc::new("".to_string()),
                    count: 0.0
                }))
        )
        .center()
}

fn main() {
    let mut data = Vector::new();
    data.push_back(AppData {
        name: Arc::new("Test".to_string()),
        count: 3.0
    });
    data.push_back(AppData {
        name: Arc::new("Test2".to_string()),
        count: 5.0
    });
    data.push_back(AppData {
        name: Arc::new("Tegrdthfztdgrsefrgdst".to_string()),
        count: 7.0
    });

    let window = WindowDesc::new(root_widget)
        .title("test table");

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(data)
        .expect("could not launch druid")
}
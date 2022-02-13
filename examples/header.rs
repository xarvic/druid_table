use std::sync::Arc;
use druid::{AppLauncher, Widget, WidgetExt, WindowDesc, Data, Lens, ArcStr};
use druid::im::Vector;
use druid::lens::Identity;
use druid::widget::{Axis, Button, Flex, Label, Slider, TextBox};
use druid_table::{HeaderTable, HeaderWidget, Static, Table};

#[derive(Clone, Data, Lens)]
struct AppData {
    name: Arc<String>,
    count: f64,
}

fn root_widget() -> impl Widget<Vector<AppData>> {

    let table = HeaderTable::new_static(Axis::Vertical, 20.0)
        .with_line(Identity, AppData::name, ||TextBox::multiline(), Label::new("Name".to_string()))
        .with_line(Identity, AppData::count, ||Slider::new().with_range(0.0, 10.0), Label::new("Value".to_string()));
    Flex::column()
        .with_flex_child(table, 1.0)
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
        name: Arc::new("Test3".to_string()),
        count: 7.0
    });

    let window = WindowDesc::new(root_widget())
        .title("test table");

    AppLauncher::with_window(window)
        .log_to_console()
        .launch(data)
        .expect("could not launch druid")
}
use masonry::widget::{Button, Flex, Label, RootWidget};
use masonry::text::StyleProperty;
use masonry::{Action, AppDriver, DriverCtx, WidgetId};

pub struct Driver {
    count: u32,
}

impl Driver {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl AppDriver for Driver {
    fn on_action(&mut self, ctx: &mut DriverCtx<'_>, _widget_id: WidgetId, action: Action) {
        match action {
            Action::ButtonPressed(_) => {
                self.count += 1;
                let mut root = ctx.get_root::<RootWidget<Flex>>();
                let mut flex = RootWidget::child_mut(&mut root);
                let new_label = Label::new(format!("Count: {}", self.count))
                    .with_style(StyleProperty::FontSize(24.0));
                Flex::remove_child(&mut flex, 0);
                Flex::add_child(&mut flex, new_label);
            }
            _ => {}
        }
    }
}

pub fn build_ui() -> impl masonry::Widget {
    Flex::column()
        .with_child(Label::new("Count: 0").with_style(StyleProperty::FontSize(24.0)))
        .with_spacer(20.0)
        .with_child(Button::new("Increment"))
        .with_spacer(20.0)
        .with_child(Button::new("Reset"))
} 
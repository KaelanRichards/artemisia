use masonry::widget::RootWidget;
use masonry::event_loop_runner::EventLoop;
use masonry::dpi::LogicalSize;
use winit::window::Window;

mod xilem_app;

fn main() {
    tracing_subscriber::fmt::init();

    let window_size = LogicalSize::new(400.0, 400.0);
    let window_builder = Window::default_attributes()
        .with_title("Solaris UI")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    masonry::event_loop_runner::run(
        EventLoop::with_user_event(),
        window_builder,
        RootWidget::new(xilem_app::build_ui()),
        xilem_app::Driver::new(),
    )
    .unwrap();
}

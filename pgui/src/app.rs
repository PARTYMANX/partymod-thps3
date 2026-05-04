pub fn run<T: 'static>(
    state: T,
    component: crate::component::Component<T>,
    window: crate::window::Window<T>,
) {
    crate::win32::app::run(state, component, window);
}

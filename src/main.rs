mod helloTriangleApplication;

fn main() {
    let mut app = helloTriangleApplication::HelloTriangleApplication::init();
    app.main_loop();
}
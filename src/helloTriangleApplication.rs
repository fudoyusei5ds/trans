extern crate gfx_backend_dx12 as backend;
extern crate gfx_hal as hal;
extern crate winit;
extern crate image;

// 设置窗口高度和宽度
const DIMS: hal::window::Extent2D = hal::window::Extent2D { width: 800,height: 600 };

#[allow(unused)]
use hal::{
    Instance,
    adapter::PhysicalDevice,
    window::Surface,
    device::Device,
    pso::DescriptorPool,
    format::AsFormat,
    window::Swapchain,
};

#[allow(unused)]
use std::io::Read;

#[allow(unused)]
pub struct HelloTriangleApplication {
    instance: backend::Instance,
    events_loop: winit::EventsLoop,
    adapter: hal::Adapter<backend::Backend>,
}

impl HelloTriangleApplication {
    pub fn init() -> Self {
        let instance = Self::create_instance();
        let (events_loop, surface) = Self::create_surface(&instance); 
        let mut adapters = instance.enumerate_adapters();
        let adapter = adapters.remove(0);

        Self {
            instance,
            events_loop,
            adapter,
        }
    }

    // 创建实例, 实例是gfx API的接口
    fn create_instance() -> backend::Instance {
        backend::Instance::create("helloworld", 1)
    }

    // 创建events_loop和surface
    fn create_surface(
        instance: &backend::Instance
    ) -> (winit::EventsLoop, hal::Backend::Surface) 
    {
        let events_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new()
            .with_dimensions(winit::dpi::LogicalSize::new(
                DIMS.width as _,
                DIMS.height as _,
            ))
            .with_title("first program".to_string())
            .build(&events_loop).unwrap();
        let mut surface = instance.create_surface(&window);
        (events_loop, surface)
    }

    // 主循环函数
    #[allow(unused)]
    pub fn main_loop(&mut self) {
        let mut running = true;
        while running {
            self.events_loop.poll_events(|event| {
                if let winit::Event::WindowEvent {event, ..} = event {
                    #[allow(unused_variables)]
                    match event {
                        winit::WindowEvent::KeyboardInput {
                            input:
                                winit::KeyboardInput {
                                    virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        }
                        | winit::WindowEvent::CloseRequested => running = false,
                        _ => (),
                    }
                }
            });
        }
    }

    // 清理函数
    #[allow(unused)]
    pub fn cleanup() {

    }
}
extern crate gfx_backend_vulkan as backend;
extern crate gfx_hal as hal;
extern crate winit;
extern crate image;

const DIMS: hal::window::Extent2D = hal::window::Extent2D { width: 800,height: 600 };
const ENTRY_NAME: &str = "main";

use hal::Instance;
use hal::adapter::PhysicalDevice;
use hal::window::Surface;
use hal::device::Device;
use hal::pso::DescriptorPool;
use hal::format::AsFormat;

// 顶点结构体
#[derive(Debug, Clone, Copy)]
struct Vertex {
    a_Pos: [f32; 2],
    a_Uv: [f32; 2],
}

// 在这里指定顶点的坐标
const QUAD: [Vertex; 6] = [
    Vertex { a_Pos: [ -0.5, 0.33 ], a_Uv: [0.0, 1.0] },
    Vertex { a_Pos: [  0.5, 0.33 ], a_Uv: [1.0, 1.0] },
    Vertex { a_Pos: [  0.5,-0.33 ], a_Uv: [1.0, 0.0] },

    Vertex { a_Pos: [ -0.5, 0.33 ], a_Uv: [0.0, 1.0] },
    Vertex { a_Pos: [  0.5,-0.33 ], a_Uv: [1.0, 0.0] },
    Vertex { a_Pos: [ -0.5,-0.33 ], a_Uv: [0.0, 0.0] },
];

const COLOR_RANGE: hal::image::SubresourceRange = hal::image::SubresourceRange {
    aspects: hal::format::Aspects::COLOR,
    levels: 0..1,
    layers: 0..1,
};


fn main() {
    // 首先创建一个物理窗口
    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(
            DIMS.width as _,
            DIMS.height as _,
        ))
        .with_title("first program".to_string())
        .build(&events_loop).unwrap();
    
    // 接着, 创建一个实例: 实例是API的接口
    let instance = backend::Instance::create("first quad", 1);
    // 创建一个表面: 表面是窗口的一种表示
    let mut surface = instance.create_surface(&window);
    // 创建一组适配器: 适配器表示一个物理设备
    let mut adapters = instance.enumerate_adapters();
    // 打印适配器的信息, 不知道有什么用
    for adapter in &adapters {
        println!("{:?}", adapter.info);
    }
    // 然后获取第一个适配器, 我们就用这个适配器来运行程序
    let mut adapter = adapters.remove(0);
    // 获取显卡的内存类型, 资源限制
    let memory_types = adapter.physical_device.memory_properties().memory_types;
    let limits = adapter.physical_device.limits();

    // 获取逻辑设备和相关的队列族, 队列族包含至少1个队列, 支持图形能力, 且和surface兼容
    let (device, mut queue_group) = adapter
        .open_with::<_, hal::Graphics>(
            1, 
            |family| surface.supports_queue_family(family),
        ).unwrap();

    // 创建一个命令池, 命令池是命令缓冲区获取内存的对象. 
    // 内存本身是隐式并动态分配的, 但如果没有它, 命令缓冲区将没有任何存储空间来保存记录的命令. 
    let mut command_pool = unsafe {
        device.create_command_pool_typed(
            &queue_group, 
            hal::pool::CommandPoolCreateFlags::empty(),
        )
    }.expect("Cannot create command pool");

    // 设置renderpass和管线

    // 创建一个描述符集合布局
    // 描述符是一个特殊的不透明的着色器变量, 着色器使用它以间接的方式访问缓冲区和图像资源. 
    // 描述符集合被称为"集合", 因为它可以引用一组同构资源, 可以用相同的布局绑定(Layout Binding)来描述.
    let set_layout = unsafe {
        device.create_descriptor_set_layout(
            &[
                hal::pso::DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: hal::pso::DescriptorType::SampledImage,
                    count: 1,
                    stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                hal::pso::DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: hal::pso::DescriptorType::Sampler,
                    count: 1,
                    stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
            ],
            &[],
        )
    }.expect("Cannot create descriptor set layout");
    // 创建描述器
    let mut desc_pool = unsafe {
        device.create_descriptor_pool(
            1,
            &[
                hal::pso::DescriptorRangeDesc {
                    ty: hal::pso::DescriptorType::SampledImage,
                    count: 1,
                },
                hal::pso::DescriptorRangeDesc {
                    ty: hal::pso::DescriptorType::Sampler,
                    count: 1,
                },
            ]
        )
    }.expect("Cannot create descriptor pool");
    // 创建描述器集合
    let desc_set = unsafe {
        desc_pool.allocate_set(&set_layout)
    }.unwrap();

    // 接下来创建顶点缓冲区
    // 首先为顶点缓冲分配内存
    println!("Memory types: {:?}", memory_types);
    // 获取顶点结构体的长度
    let buffer_stride = std::mem::size_of::<Vertex>() as u64;
    // 计算顶点缓冲区的大小
    let buffer_len = QUAD.len() as u64 * buffer_stride;
    // 断言长度不为0
    assert_ne!(buffer_len, 0);
    // 创建顶点缓冲(未绑定内存)
    let mut vertex_buffer = unsafe {
        device.create_buffer(
            buffer_len,
            hal::buffer::Usage::VERTEX,
        )
    }.unwrap();
    // 获取顶点缓冲区所需的内存
    let buffer_req = unsafe {
        device.get_buffer_requirements(&vertex_buffer)
    };
    // 获取可用的内存类型
    let upload_type = memory_types
        .iter()
        .enumerate()
        .position(
            // type_mask是一个位字段, 每位表示一种内存类型
            // 如果位设置为1, 说明该内存可以用于缓冲区
            // 因此, 查找第一个位为1, 且对CPU可见的内存类型
            |(id, mem_type)| {
                buffer_req.type_mask & (1 << id) != 0
                   && mem_type.properties.contains(hal::memory::Properties::CPU_VISIBLE)
            }
        ).unwrap().into();
    // 为缓冲区分配指定类型的内存段
    let buffer_memory = unsafe {
        device.allocate_memory(
            upload_type,
            buffer_req.size, // 顶点缓冲区的内存大小
        )
    }.unwrap();
    // 把内存绑定到顶点缓冲区
    unsafe {
        device.bind_buffer_memory(
            &buffer_memory,
            0,
            &mut vertex_buffer,
        )
    }.unwrap();
    // 写数据到顶点缓冲区
    unsafe {
        // 首先获取一个写入内存的映射
        let mut vertices = device
            .acquire_mapping_writer::<Vertex>(&buffer_memory, 0..buffer_req.size)
            .unwrap();
        // 然后将顶点复制到映射中
        vertices[0..QUAD.len()].copy_from_slice(&QUAD);
        // 释放写映射
        device.release_mapping_writer(vertices).unwrap();
    }


    // 处理图片, 将图片作为纹理上传到uniform变量中
    // 首先将图片保存为二进制数据
    let img_data = include_bytes!("data/logo.png");
    // 用image模块读取图片
    let img = image::load(std::io::Cursor::new(&img_data[..]), image::PNG)
        .unwrap().to_rgba();
    // 获取图片的宽高
    let (width, height) = img.dimensions();
    // 指定将分配的图片的类型
    let kind = hal::image::Kind::D2( // 二维图像
        width as u32,
        height as u32,
        1,  // 图层数
        1,  // 采样数
    );
    // 获取行对齐掩码, 值为存储在缓冲区中的纹理数据的行间距的对齐(主要用于GPU复制数据)减1
    let row_alignment_mask = limits.min_buffer_copy_pitch_alignment as u32 - 1;
    // 这是什么意思?
    let image_stride = 4usize;
    // 计算行距
    let row_pitch = 
        (width * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
    // 计算保存图像的缓冲区的大小
    let upload_size = (height * row_pitch) as u64;
    // 创建用于保存图片的缓冲区
    let mut image_upload_buffer = unsafe {
        device.create_buffer(
            upload_size,  
            hal::buffer::Usage::TRANSFER_SRC    // 该缓冲区用来作为转换源
        )
    }.unwrap();
    // 获取缓冲区需求的内存
    let image_mem_reqs = unsafe {
        device.get_buffer_requirements(&image_upload_buffer)
    };
    // 为该缓冲区分配内存
    let image_upload_memory = unsafe {
        device.allocate_memory(
            upload_type, 
            image_mem_reqs.size)
    }.unwrap();
    // 然后把内存绑定到缓冲区上
    unsafe {
        device.bind_buffer_memory(
            &image_upload_memory,
            0,
            &mut image_upload_buffer,
        )
    }.unwrap();
    // 最后, 把图片的数据复制到现在的缓冲区上
    unsafe {
        let mut data = device
            .acquire_mapping_writer::<u8>(
                &image_upload_memory,
                0..image_mem_reqs.size
            ).unwrap();
        for y in 0..height as usize {
            let row = &(*img)
                [y * (width as usize) * image_stride..(y + 1) * (width as usize) * image_stride];
            let dest_base = y * row_pitch as usize;
            data[dest_base..dest_base + row.len()].copy_from_slice(row);
        }
        device.release_mapping_writer(data).unwrap();
    }
    // 下面创建一个纹理
    // 首先创建一个图片对象
    let mut image_logo = unsafe {
        device.create_image(
            kind,       // 类型
            1,          // 多级渐远纹理等级
            hal::format::Rgba8Srgb::SELF,   // 格式
            hal::image::Tiling::Optimal,            // 平铺
            hal::image::Usage::TRANSFER_DST | 
                hal::image::Usage::SAMPLED,         // 使用标记
            hal::image::ViewCapabilities::empty(),  // 不懂
        )
    }.unwrap();
    // 获取该图片对象的内存需求
    let image_req = unsafe {
        device.get_image_requirements(&image_logo)
    };
    // 获取支持图片对象的设备内存的类型
    let device_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, memory_type)| {
            image_req.type_mask & (1 << id) != 0
                && memory_type.properties.contains(hal::memory::Properties::CPU_VISIBLE)
        }).unwrap().into();
    // 分配内存
    let image_memory = unsafe {
        device.allocate_memory(
            device_type,
            image_req.size)
    }.unwrap();
    // 绑定内存
    unsafe {
        device.bind_image_memory(
            &image_memory,
            0,
            &mut image_logo,
        )
    }.unwrap();
    // 使用一张已有的图片创建一个image view
    let image_srv = unsafe {
        device.create_image_view(
            &image_logo,                    // 源图像
            hal::image::ViewKind::D2,       // 类型
            hal::format::Rgba8Srgb::SELF,   // 格式
            hal::format::Swizzle::NO,       // 是否将图像映射为其他格式
            COLOR_RANGE.clone(),            // 这个参数有什么用
        )
    }.unwrap();
    // 创建一个采样器对象
    let sampler = unsafe {
        device.create_sampler(
            hal::image::SamplerInfo::new(
                hal::image::Filter::Linear, // 设置采样纹理的过滤方式 
                hal::image::WrapMode::Clamp,
            )
        )
    }.expect("Cannot create sampler");
    // 指定描述符集的写入操作的参数
    unsafe {
        device.write_descriptor_sets(vec![
            // 将要绑定的实际描述符写入描述符集合
            hal::pso::DescriptorSetWrite {
                set: &desc_set,
                binding: 0,
                array_offset: 0,
                descriptors: Some(
                    hal::pso::Descriptor::Image(&image_srv, hal::image::Layout::Undefined)
                ),
            },
            hal::pso::DescriptorSetWrite {
                set: &desc_set,
                binding: 1,
                array_offset: 0,
                descriptors: Some(
                    hal::pso::Descriptor::Sampler(&sampler)
                ),
            }
        ]);
    }
    // 将缓冲区复制到纹理中
    // 首先创建一个fence信号. 
    let mut copy_fence = device.create_fence(false).expect("Cannot create fence");
    unsafe {
        // 创建一个只用一次的命令缓冲
        let mut cmd_buffer = command_pool.acquire_command_buffer::<hal::command::OneShot>();
        // 开始记录命令缓冲
        cmd_buffer.begin();
        // 为图片创建一个内存屏障
        // 内存屏障在移动或修改内存时使用, 具体用处还不知道
        let image_barrier: hal::memory::Barrier<backend::Backend> = hal::memory::Barrier::Image {
            states: (hal::image::Access::empty(), hal::image::Layout::Undefined)
                ..(hal::image::Access::TRANSFER_WRITE, hal::image::Layout::TransferDstOptimal),
            target: &image_logo,
            families: None,
            range: COLOR_RANGE.clone(),
        };
        // 在命令缓冲区的管道阶段之间插入同步依赖项
        cmd_buffer.pipeline_barrier(
            hal::pso::PipelineStage::TOP_OF_PIPE
                ..hal::pso::PipelineStage::TRANSFER,
            hal::memory::Dependencies::empty(),
            &[image_barrier],
        );
        // 从缓冲区复制内容到图片
        cmd_buffer.copy_buffer_to_image(
            &image_upload_buffer,   // 源
            &image_logo,            // 目标
            hal::image::Layout::TransferDstOptimal, // 目标布局
            &[hal::command::BufferImageCopy {   // 指定复制缓冲区到图片的所有参数
                buffer_offset: 0,
                buffer_width: row_pitch / (image_stride as u32),
                buffer_height: height as u32,
                image_layers: hal::image::SubresourceLayers {
                    aspects: hal::format::Aspects::COLOR,
                    level: 0,
                    layers: 0..1,
                },
                image_offset: hal::image::Offset {
                    x: 0, 
                    y: 0, 
                    z: 0
                },
                image_extent: hal::image::Extent {
                    width,
                    height,
                    depth: 1,
                },
            }],
        );
        // 然后再把图片上传到着色器
        let image_barrier: hal::memory::Barrier<backend::Backend> = hal::memory::Barrier::Image {
            states: (hal::image::Access::TRANSFER_WRITE, hal::image::Layout::TransferDstOptimal)
                ..(hal::image::Access::SHADER_READ, hal::image::Layout::ShaderReadOnlyOptimal),
            target: &image_logo,
            families: None,
            range: COLOR_RANGE.clone(),
        };
        cmd_buffer.pipeline_barrier(
            hal::pso::PipelineStage::TRANSFER
                ..hal::pso::PipelineStage::FRAGMENT_SHADER,
            hal::memory::Dependencies::empty(),
            &[image_barrier],
        );
        // 完成命令记录
        cmd_buffer.finish();
        // 将命令缓冲区提交到队列族中
        queue_group.queues[0].submit_nosemaphores(
            Some(&cmd_buffer), 
            Some(&mut copy_fence),
        );
        device 
            .wait_for_fence(&copy_fence, !0)    // 等待命令执行完毕
            .expect("Cannot wait for fence");
    }
    // 删除fence信号
    unsafe {
        device.destroy_fence(copy_fence);
    }


    // 获取surface兼容性和surface的格式
    let (caps, formats, _present_modes, _composite_alpha) = 
        surface.compatibility(&mut adapter.physical_device);
    // 打印
    println!("formats: {:?}", formats);
    // 从所有支持的格式中, 选择srgb格式
    let format = formats.map_or(
        hal::format::Format::Rgba8Srgb,
        |formats| {
            formats
                .iter()
                .find(
                    |format| format.base_format().1 == hal::format::ChannelType::Srgb
                ).map(|format| *format)
                .unwrap_or(formats[0])
        }
    );
    // 创建交换链配置
    let swap_config = hal::window::SwapchainConfig::from_caps(
        &caps, 
        format, 
        DIMS
    );
    println!("{:?}", swap_config);
    // 获取交换区图像尺寸
    let extent = swap_config.extent.to_extent();
    // 创建交换链和backbuffer
    let (mut swap_chain, mut backbuffer) = unsafe {
        device.create_swapchain(
            &mut surface,
            swap_config,
            None,
        )
    }.expect("Cannot create swapchain");
    // 创建renderpass
    let render_pass = {
        // 首先创建一个附件
        let attachment = hal::pass:Attachment {
            format: Some(format),
            samples: 1,
            ops: hal::pass::AttachmentOps::new(
                hal::pass::AttachmentLoadOp::Clear,
                hal::pass::AttachmentStoreOp::Store,
            ),
            stencil_ops: hal::pass::AttachmentOps::DONT_CARE,
            layout: hal::image::Layout::Undefined
                ..hal::image::Layout::Present,
        };
        // 创建一个子过程
        let subpass = hal::pass::SubpassDesc {
            colors: &[(0, hal::image::Layout::ColorAttachmentOptimal)],
            depth_stencil: None,
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };
        // 接着, 指定多个子过程之间的依赖关系
        let dependency = hal::pass::SubpassDependency {
            passes: hal::pass::SubpassRef::External
                ..hal::pass::SubpassRef::Pass(0),
            stages: hal::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT
                ..pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            accesses: hal::image::Access::empty()
                ..(hal::image::Access::COLOR_ATTACHMENT_READ | hal::imgae::Access::COLOR_ATTACHMENT_WRITE),
        };
        // 最后, 创建render pass
        unsafe {
            device.create_render_pass(
                &[attachment],
                &[subpass],
                &[dependency],
            )
        }.expect("Cannot create render pass")
    };
    // 给交换链中的每个图像创建一个imgaeview和帧缓冲
    let (mut frame_images, mut framebuffers) = match backbuffer {
        hal::Backbuffer::Images(images) => {
            let pairs = images
                .into_iter()
                .map(|image| unsafe {
                    let rtv = device.create_image_view(
                        &image,
                        hal::image::ViewKind::D2,
                        format,
                        hal::format::Swizzle::NO,
                        COLOR_RANGE.clone(),
                    ).unwrap();
                    (image, rtv)
                })
                .collect::<Vec<_>>();
            let fbos = pairs
                .iter()
                .map(|&(_, ref rtv)| unsafe {
                    device.create_framebuffer(
                        &render_pass, 
                        Some(rtv),
                        extent
                    ).unwrap()
                })
                .collect();
            (pairs, fbos)
        }
        hal::Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
    };
}
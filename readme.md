# 写一个三角形

### 实例

首先, 用 winit 创建一个物理窗口:  

```
let mut events_loop = EventsLoop::new();
let window = WindowBuilder::new()
    .with_title("Part 00: Triangle")
    .with_dimensions((800, 600).into())
    .build(&events_loop)
    .unwrap();
```

然后, 需要创建一个实例, 实例是福尔康库和应用程序之间的连接, 创建的时候, 需要把应用程序的信息传递给驱动程序:  

```
// create的这两个参数其实并不重要
let instance = backend::Instance::create("Part 00: Triangle", 1);
```

将物理窗口和福尔康库直接联系起来:  

```
let mut surface = instance.create_surface(&window);
```

### 创建设备

在初始化实例之后, 我们需要在系统中找出支持我们所需功能的显卡. 我们可以同时使用多张显卡, 不过在这里只使用一个:  

```
// adapter就是一个物理设备
let mut adapter = instance.enumerate_adapters().remove(0);
```

在福尔康中, 几乎每一个操作, 从绘制到上传纹理, 都需要将命令提交到命令队列中, 不同类型的命令队列属于不同的队列族, 而每个队列族只支持一类命令: 例如, 计算命令, 传输命令, 图形命令等.  

我们要找出支持我们要使用的命令的设备:  
同时, 尽管福尔康的实现支持窗口系统, 不过不是每个设备都支持, 我们需要确保使用的设备具备向我们创建的surface绘制图像的功能. 因为显示是队列指定的功能, 因此查看队列族是否支持向surface绘制图像.  

```
// num_queues 为需要创建的队列数, 一般为1
let num_queues = 1;
// 使用 device 来保存我们获取的物理设备, queue_group 保存命令队列, 稍后我们可以向它提交绘制命令
let (device, mut queue_group) = adapter
    .open_with::<_, Graphics>(num_queues, |family| surface.supports_queue_family(family))
    .unwrap();
```

### 交换链

福尔康中没有默认缓冲区的概念, 因此我们需要创建一个基础结构, 用来在将缓冲区绘制到屏幕上之前保存缓冲区. 这个结构被称为 swap chain , 在福尔康中, 必须被显示创建.  交换链本质上是一个等待绘制到屏幕上的图像的队列. 我们的应用程序将获取这样一个图像来绘制它, 然后将其返回到队列中. 队列的工作方式和显示队列中图像的条件取决于交换链的设置方式, 但交换链的一般用途是将图像的显示与屏幕的刷新率同步. 

创建交换链首先要检查其与我们的窗口surface是否兼容, 需要获取更多的信息才能开始创建交换链:  

```
let (
    caps,               // 基础的surface兼容性, 例如交换链中的图像数量最大值和最小值, 以及图像的最大和最小宽高
    formats,            // surface的格式, 例如像素格式, 颜色空间等
    _present_modes,     // 可用的显示模式
    _composite_alpha    // alpha 混合? 不懂
    ) = surface.compatibility(&mut adapter.physical_device);
```

接下来就是为交换链设置配置, 一共需要设置以下三种配置:  

1. surface格式(颜色, 深度)
2. 显示模式(显示图像的条件)
3. 交换程度(交换链中图像的分辨率)  

首先获取surface格式:  

```
// base_format() 获取的颜色第一个为surface颜色, 第二个为channel颜色
// 我们需要将颜色空间指定为srgb, 为此, 需要将channel颜色指定为标准rgba8
let surface_color_format = {
    match formats {
        Some(choices) => choices
            .into_iter()
            .find(|format| format.base_format().1 == ChannelType::Srgb)
            .unwrap(),
        None => Format::Rgba8Srgb,
    }
};
```

显示模式可以说是交换链中最重要的设置了, 因为它表示实际显示图片的条件, 福尔康有四种可能的显示模式:  

1. VK_PRESENT_MODE_IMMEDIATE_KHR: 立即显示图像, 可能会导致画面撕裂.  
2. VK_PRESENT_MODE_FIFO_KHR: 交换链变成一个队列, 当显示刷新并且程序在队列末尾插入图像时, 显示将从队列的前面获取图像. 如果队列已满, 则程序必须等待. 这与现代游戏中的垂直同步最为相似. 刷新显示的时刻称为"垂直空白".  
3. VK_PRESENT_MODE_FIFO_RELAXED_KHR: 只有当应用程序延迟并且队列在最后一个垂直空白处为空时, 此模式才与前一个模式不同. 图像不会等待下一个垂直空白, 而是在最终到达时立即传输. 这可能导致可见撕裂.  
4. VK_PRESENT_MODE_MAILBOX_KHR: 这是第二种模式的另一种变化. 当队列已满时, 不用阻塞应用程序, 只需用较新的映像替换已排队的映像. 此模式可用于实现三重缓冲, 与使用双缓冲的标准垂直同步相比, 它允许您避免延迟问题显著减少的中断. 

默认使用的是第二种模式.   

交换程度是交换链图像的分辨率, 它几乎总是与我们要绘制的窗口的分辨率完全相同. 

接下来创建交换链:  

```
let swap_config = SwapchainConfig::from_caps(
        &caps,                          // 根据surface的兼容性来创建, 不知道有什么用
        surface_color_format,           // surface的通道格式
        gfx_hal::window::Extent2D {     // 如果surface没有指定宽高, 那么就用这个宽高
            width: 800,
            height: 600,
        }
);
// backbuffer 中包含组成交换链的实际图像
let (mut swapchain, backbuffer) = unsafe {
        device.create_swapchain(&mut surface, swap_config, None)
}.unwrap();
```

### 管线

渲染管线和opengl类似.  

*input assembler* 从指定的缓冲收集原始数据, 也可以使用索引缓冲来重复绘制一些顶点, 而不必复制顶点数据本身.  

顶点着色器对每个顶点运行, 将顶点从模型空间转化到屏幕空间. 同时通过管线传递数据.  

细分着色器可以根据规则细分几何体, 提高网格质量. 通常用于地面或者墙壁.  

几何着色器用的少, 因为其硬件支持不是很好.  

光栅化阶段将图元拆分成片段, 丢弃屏幕外的片段. 同时对顶点着色器的输出进行插值. 同时, 进行深度测试.  

片段着色器在每个留下来的片段上调用, 并且确定片段写入哪个帧缓冲, 以及使用什么颜色和深度.  

颜色混合阶段混合映射到帧缓冲的同一个像素的不同片段.  

福尔康中的管线是不可变的, 如果想修改着色器, 使用另一个帧缓冲, 换一个混合函数, 你都要从头创建一个管线.  

### 着色器模型

```
let vertex_shader_module = {
    let glsl = fs::read_to_string("part00.vert").unwrap();
    let spirv: Vec<u8> = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Vertex)
            .unwrap()
            .bytes()
            .map(|b| b.unwrap())
            .collect();
    unsafe {
        device.create_shader_module(&spirv)
    }.unwrap()
};
let fragment_shader_module = {
    let glsl = fs::read_to_string("part00.frag").unwrap();
        let spirv: Vec<u8> = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Fragment)
            .unwrap()
            .bytes()
            .map(|b| b.unwrap())
            .collect();
    unsafe {
        device.create_shader_module(&spirv)
    }.unwrap()
};
```

接下来将这些模块分配到管道的各个阶段中.  

```
let vs_entry = EntryPoint::<backend::Backend> {
    entry: "main",                      // 指定入口函数
    module: &vertex_shader_module,      // 着色器模块
    specialization: Default::default(), // 特化常量, 为着色器中使用的常量指定值, 相当于外部宏定义这种
};
// 片段着色器就不列举了
```

指定各个着色器, 没用到的就None掉. 

```
let shader_entries = GraphicsShaderSet {
    vertex: vs_entry,
    hull: None,
    domain: None,
    geometry: None,
    fragment: Some(fs_entry),
};
```

### 固定函数

opengl为大多数阶段提供了默认状态, 而福尔康不同, 必须确定渲染管线的每一步.  

### Render passes

在完成渲染管道的创建之前, 我们需要通知福尔康, 帧缓冲的附件有哪些. 我们需要为帧缓冲区指定颜色和深度缓冲, 以及它们的采样数, 以及如何在渲染操作中处理它们的内容. 

创建颜色附件:  

```
let color_attachment = Attachment {
    format: Some(surface_color_format), // 附件的格式
    samples: 1,                         // 采样数
    ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store), // 加载和储存操作
    stencil_ops: AttachmentOps::DONT_CARE,          // 加载和储存模板测试的操作
    layouts: Layout::Undefined..Layout::Present,    // render pass 最初和最终的图像布局
};
```

一个render pass可以由很多个子过程组成, 子过程会根据前面过程中帧缓冲区的内容后续执行操作. 例如一系列相继应用的后处理效果. 将这些渲染操作分到一个render pass, 福尔康可以对内存带宽进行优化.  

每个子过程都会使用一个或者多个附件.  

```
let subpass = SubpassDesc {
    colors: &[(0, Layout::ColorAttachmentOptimal)], // 所使用的颜色附件
    depth_stencil: None,                            // 所使用的深度测试或者模板测试附件
    inputs: &[],                                    // 将哪个附件作为该子过程的输入
    resolves: &[],                                  // 用于多重采样的颜色附件
    preserves: &[],                                 // 此子过程未使用, 但必须保存其数据的附件
};
```

多个子过程之间的依赖关系:  

```
// 这里不太清楚其工作的原理
let dependency = SubpassDependency {
    passes: SubpassRef::External..SubpassRef::Pass(0),
    stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
    accesses: Access::empty()
        ..(Access::COLOR_ATTACHMENT_READ | Access::COLOR_ATTACHMENT_WRITE),
};
```

接着创建render pass

```
unsafe {
    device.create_render_pass(&[color_attachment], &[subpass], &[dependency])
}.unwrap()
```

### 管线布局

你可以在着色器中使用uniform变量, 这是一种全局变量, 可以在绘制时使用, 你也可以直接修改它们的值而无需重新创建着色器. 通常用于传递变换矩阵或者纹理. 

push constant是传递值给着色器程序的另一种方式.  

```
// 着色器所使用的uniform值和push值, 可以在绘制时传递给着色器
let pipeline_layout = unsafe {
    device.create_pipeline_layout(&[], &[])
}.unwrap();
```

### 创建管道

```
// 创建图形管道的所有设置的描述
let mut pipeline_desc = GraphicsPipelineDesc::new(
    shader_entries,             // 着色器集
    Primitive::TriangleList,    // 描述了从顶点创建图形的基本类型
    Rasterizer::FILL,           // 光栅化阶段
    &pipeline_layout,           // 管道布局
    subpass,                    // render pass的一个引用
);
```

```
// 这里是做什么用的?
pipeline_desc
    .blender
    .targets
    .push(ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA));
```

```
// 最后创建
unsafe {
    device.create_graphics_pipeline(&pipeline_desc, None)
}.unwrap()
```

### 帧缓冲

我们已经设置好了渲染过程, 以期望使用与交换链图像相同格式的单个帧缓冲区. 

```
// 需要为交换链中的每个图像创建帧缓冲区和imageview
let (frame_views, framebuffers) = match backbuffer {
    Backbuffer::Images(images) => {
        // 包含在图像中的资源的子集
        let color_range = SubresourceRange {
            // 被包含的种类
            aspects: Aspects::COLOR,
            // Included mipmap levels
            levels: 0..1,
            // Included array levels
            layers: 0..1,
        };

        let image_views = images
            .iter()
            .map(|image| {
                unsafe {
                    device.create_image_view(
                        image,                  // 图像
                        ViewKind::D2,           // 类型
                        surface_color_format,   // 格式
                        Swizzle::NO,            // 不懂
                        color_range.clone(),    // 不懂
                    )   
                }.unwrap()
            })
            .collect::<Vec<_>>();

        let fbos = image_views
            .iter()
            .map(|image_view| {
                unsafe {
                    device.create_framebuffer(
                        &render_pass,       
                        vec![image_view],   // 附件
                        extent              // 缓冲区尺寸
                    )
                }.unwrap()
            })
            .collect();

        (image_views, fbos)
    }

    Backbuffer::Framebuffer(fbo) => (vec![], vec![fbo]),
};
```

### 命令缓冲

福尔康中的命令, 如绘制和传输, 不能直接使用函数调用执行, 必须保存到命令缓冲区中. 

### 命令池

我们必须先创建一个命令池, 命令池用于管理存储缓冲区的内存, 并从中分配命令缓冲区.  

命令缓冲区是在一个设备队列上提交来执行的, 比如我们检测到的图形和表示队列. 每个命令池只能分配在单个队列类型上提交的命令缓冲区. 我们将记录绘图命令，这就是为什么我们选择了图形队列族. 

```
let num_queues = 1;
let (device, mut queue_group) = adapter
    .open_with::<_, Graphics>(num_queues, |family| surface.supports_queue_family(family))
    .unwrap();

let mut command_pool = unsafe {
    device.create_command_pool_typed(       // 创造一个强类型的命令池
        &queue_group,                       // 
        CommandPoolCreateFlags::empty(),)   // 命令池有两种可能的flag:
// TRANSIENT: 指示命令池经常会使用新的命令, 需要经常进行重新编码
// RESET_INDIVIDUAL: 运行命令池单独重新编码? 
}.expect("Can't create command pool");
```

### 分配命令缓冲区

现在开始分配命令缓冲区, 并在其中提交绘图命令. 我们需要为交换链中的每个图像记录一个命令缓冲区.  

```
// 分配一个主缓冲区. 
let mut command_buffer = command_pool.acquire_command_buffer::<gfx_hal::command::MultiShot>();
```

### 开始

```
command_buffer.set_viewports(0, &[viewport.clone()]);   // 设置光栅器的视区参数
command_buffer.set_scissors(0, &[viewport.rect]);       // 设置光栅器的裁剪参数

// Choose a pipeline to use.
command_buffer.bind_graphics_pipeline(&pipeline);       // 选择使用的渲染管线
```


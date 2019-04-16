
首先, 我们必须设置一组渲染状态. 我们将需要以下东西:   

    一个窗口.  
    一个实例, 设备, 适配器, 和各种各样其他的东西.  
    一个renderpass, 用来定义如何使用不同的图像.  
    一个管线, 包括我们的着色器. 它定义了我们怎么渲染.  
    一个交换链, 它是一组将被渲染, 并展示到屏幕上的图像.   
    在交换链中每个图像都有一个image view和一个帧缓冲. 二者允许我们把指定交换链中的图像绑定到我们的renderpass上.   

然后在每一帧上, 我们都将通过以下几步渲染我们的三角形:  

    首先, 创建一个命令缓冲表示我们想要进行渲染.  
    将命令缓冲提交到命令队列中, 这将把它渲染到一个交换链的图像中.  
    然后我们展示交换链图像, 并且释放旧的图像来进行渲染.  


### 初始化

首先, 初始化窗口

```
let mut events_loop = EventsLoop::new();

let window = WindowBuilder::new()
    .with_title("Part 00: Triangle")
    .with_dimensions((800, 600).into())
    .build(&events_loop)
    .unwrap();

let instance = backend::Instance::create("Part 00: Triangle", 1);
let mut surface = instance.create_surface(&window);
let mut adapter = instance.enumerate_adapters().remove(0);

let num_queues = 1;
let (device, mut queue_group) = adapter
    .open_with::<_, Graphics>(num_queues, |family| surface.supports_queue_family(family))
    .unwrap();

let mut command_pool = unsafe {
    device.create_command_pool_typed(
        &queue_group,
        CommandPoolCreateFlags::empty(),)
}.expect("Can't create command pool");
```

window 和 events_loop 都是winit包的一部分.  

instance 用来初始化API让我们可以使用我们需要的任何东西, 包括 surface, 它是我们将要在上面绘制图形的窗口的表示.  

adapter 表示一个物理设备. 例如, 一张显卡. 在上面的代码中, 我们只使用列表中的第一个设备.  

接下来我们获取了一个 device 和一个 queue group. device 在这里表示一个逻辑设备而不是物理设备. 这是一个负责分配和释放资源的抽象概率.  

queue_group 是 command queues 的集合, 在渲染时, 你要将 command buffers 提交到 command queues 上. open_with函数的意思是: "给我一个支持图形能力的队列组, 包含至少1个队列, 并且它必须和我的 surface 兼容".  

The command_pool is where we get command buffers from in the first place, which we can then submit to a queue.
command_pool 是我们首先从中获取命令缓冲区的地方，然后我们可以将其提交到队列。


### 渲染管线

一个管线状态对象包含几乎你需要的所有状态, 为了绘制某物. 它包括着色器, 图元类型, 混合类型, 等等.  

它也包含一个 render_pass, 所以先创建一个:  

```
    let render_pass = {
        let color_attachment = Attachment {
            format: Some(surface_color_format),
            samples: 1,
            ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
            stencil_ops: AttachmentOps::DONT_CARE,
            layouts: Layout::Undefined..Layout::Present,
        };

        let subpass = SubpassDesc {
            colors: &[(0, Layout::ColorAttachmentOptimal)],
            depth_stencil: None,
            ...
        };

        let dependency = SubpassDependency {
            ...
        };

        unsafe {
            device.create_render_pass(&[color_attachment], &[subpass], &[dependency])
        }.unwrap()
    };
```

一个 render pass 定义了我们需要多少个图像("附件")用于渲染, 并且它们将怎样使用. 在本例中, 我们只关心一个图像, 就是我们要渲染的那个. 每个 render pass 包含至少一个 subpass - 在上面的代码中, 我们用了一个颜色附件, 但是没有用到深度附件.  

接下来创建一些传递给管线的着色器模型:  

```
    let vertex_shader_module = {
        let glsl = fs::read_to_string("/home/tet/test_sets/workspace/aboarpython/gfx_test/src/part00.vert").unwrap();
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
        let glsl = fs::read_to_string("/home/tet/test_sets/workspace/aboarpython/gfx_test/src/part00.frag").unwrap();
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

最后, 创建管线:  

```
    let pipeline_layout = device.create_pipeline_layout(&[], &[]);

    let pipeline = {
        let vs_entry = EntryPoint::<backend::Backend> {
            entry: "main",
            module: &vertex_shader_module,
            specialization: &[],
        };

        let fs_entry = ...;

        let shader_entries = GraphicsShaderSet {
            vertex: vs_entry,
            fragment: Some(fs_entry),
            ...
        };

        let subpass = Subpass { index: 0, main_pass: &render_pass };

        let mut pipeline_desc = GraphicsPipelineDesc::new(
            shader_entries,
            Primitive::TriangleList,
            Rasterizer::FILL,
            &pipeline_layout,
            subpass,
        );

        pipeline_desc.blender.targets.push(ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA));

        device.create_graphics_pipeline(&pipeline_desc, None).unwrap()
    };
```

重要的部分是 pipeline_desc 结构体. 正如你所见, 它包括我们的着色器, 图元类型, 光栅化类型, 管道布局, 以及一个 render pass. 同时在构建之后, 设置其混合模式.  

现在, 我们以及定义好了我们的渲染, 最后一件事是我们将在哪里渲染.  


### 交换链和帧缓冲

特别的, 我们想要在显示一个图片的同时渲染一个图片. 当我们渲染完了之后, 我们将交换二者.  

这两张图片就组成了一个交换链. 现在我们创建交换链:  

```
    let (mut swapchain, backbuffer) = {
        let extent = {
            let (width, height) = window_size;
            Extent2D { width, height }
        };

        let swap_config = SwapchainConfig::new()
            .with_color(surface_color_format)
            .with_image_usage(image::Usage::COLOR_ATTACHMENT);

        device.create_swapchain(&mut surface, swap_config, None, &extent)
    };
```

首先我们指定图像的格式, 我们打算将这些图像用于显示颜色. 同时我们还要指定我们的窗口的范围. 上面的代码将返回一个交换链, 同时返回一个 backbuffer, 它实际上包含了被交换链所使用的图像列表.  

为了获取图像的内容, 我们需要为每个图像设置 image_view. 一个 image view 可以表示一张完整图像中的一小部分, 不过这里我们打算使用完整的图像.  

同时创建帧缓冲. 记得我们定义了一个 render pass , 它描述了我们打算使用多少张图像用于渲染, 并且每张图像用于什么目的? 一个帧缓冲将绑定一个指定的 image view 到你的 render pass 中的一个指定的附件上:  

```
    let (frame_views, framebuffers) = match backbuffer {
        Backbuffer::Images(images) => {
            let (width, height) = window_size;
            let extent = Extent { width, height, depth: 1 };

            let color_range =
                SubresourceRange { aspects: Aspects::COLOR, levels: 0..1, layers: 0..1 };

            let image_views = images.iter()
                .map(|image| {
                    device.create_image_view(
                        image,
                        ViewKind::D2,
                        surface_color_format,
                        Swizzle::NO,
                        color_range.clone(),
                    ).unwrap()
                })
                .collect::<Vec<_>>();

            let fbos = image_views.iter()
                .map(|image_view| {
                    device.create_framebuffer(&render_pass, vec![image_view], extent).unwrap()
                }).collect();

            (image_views, fbos)
        }
        ...
    };
```

现在我们正要做的就是循环使用 backbuffer 中的图像创建 image views, 然后循环使用 image views 去创建帧缓冲.  

注意, 当我们创建帧缓冲时, 我们指定一个 render pass, 并且一组 image views 绑定到它上面.  

最后一件事, 我们需要一对同步源语. 它们允许我们确保我们总是渲染到一张没有显示到屏幕的图像上.  

```
    let frame_semaphore = device.create_semaphore();
    let frame_fence = device.create_fence(false);
```

所有的设置已经完成. 现在可以开始渲染了.  


### 渲染一帧

这是最简单的一部分了. 我们已经了解了渲染过程中的所有部分. 剩下的就是创建一个命令缓冲并且在渲染时提交它.  

先创建我们的命令缓冲:  

```
        let frame_index = swapchain.acquire_image(FrameSync::Semaphore(&frame_semaphore)).unwrap();

        let finished_command_buffer = {
            let viewport = Viewport {
                rect: Rect { x: 0, y: 0, w: window_width, h: window_height },
                depth: 0.0..1.0,
            };

            let mut command_buffer = command_pool.acquire_command_buffer(false);
            command_buffer.set_viewports(0, &[viewport.clone()]);
            command_buffer.set_scissors(0, &[viewport.rect]);
            command_buffer.bind_graphics_pipeline(&pipeline);
            {
                let mut encoder = command_buffer.begin_render_pass_inline(
                    &render_pass,
                    &framebuffers[frame_index as usize],
                    viewport.rect,
                    &[ClearValue::Color(ClearColor::Float([0.0, 0.0, 0.0, 1.0]))],
                );
                encoder.draw(0..3, 0..1);
            }
            command_buffer.finish()
        };
```

首先, 我们选择渲染到交换链中的哪一张图像上. 同时通知它在图像渲染完毕后发出 frame_semaphore 信号.  

然后, 从 command pool 中获取一个新的缓冲. 同时设置 viewport 和 scissor rect (我们可以选择渲染到它的某个较小的子区域中). 然后我们选择使用的管线 - 我们只有一个.  

接下来, 我们开始 render pass. 现在开始把渲染命令记录到渲染缓冲中. 我们传递我们现在的帧缓冲, 绘制区域以及一个将帧清空为黑色的指令.  

Now for the triangle itself. That draw command says “draw the first 3 vertices of the first 1 instances”. (Ignore that last part - we’re not using instanced rendering for this tutorial.) The vertex data itself, as mentioned, comes from our vertex shader this time, so this is all we need.
对于三角形来说. 绘制命令指示: "绘制第一个实例的前三个顶点". (忽略最后一个部分, 我们在本指南中没有使用实例渲染). 顶点数据都保存在顶点着色器中, 所以这就是我们所需要做的了.  

最后, 我们完成记录命令缓冲, 并准备提交.  

我们首先等待frame_semaphore信号, 这样我们的目标图像就准备好了, 然后为命令缓冲创建一个submission:  

```
        let submission = Submission::new()
            .wait_on(&[(&frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)])
            .submit(vec![finished_command_buffer]);
```

然后我们可以把它提交到我们的命令队列中, 同时通知它在渲染完毕后发送frame_fence信号, 并且等待:  

```
        queue_group.queues[0].submit(submission, Some(&frame_fence));

        device.wait_for_fence(&frame_fence, !0);
```

最后, 我们可以看到我们的三角形:  

```
        swapchain.present(&mut queue_group.queues[0], frame_index, &[]).unwrap();
```
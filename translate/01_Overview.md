本节将首先介绍福尔康以及它能解决的问题. 在此之后, 我们打算了解绘制第一个三角形需要哪些因素. 这将让你对之后的章节有一个全面的认识. 最后, 我们将介绍福尔康的API的结构以及其一般的用法.  

## 福尔康的起源  

就像之前的图形API一样, 福尔康被设计为基于[GPUs](https://en.wikipedia.org/wiki/Graphics_processing_unit)的跨平台抽象. 大多数API的问题在于在它们被设计出来的年代, 以图形为特色的硬件可配置的功能都是固定的, 局限性很大. 程序员必须提供标准格式的顶点数据, 而关于光照和着色的选项则完全由GPU厂商决定.  

随着显卡架构的成熟, 显卡开始提供越来越多的可编程功能. 所有的新功能都要以某种方式添加到现有的API中. 这就导致了API的抽象程度不够理想, 显卡驱动被迫进行大量猜测以将程序员的意图映射到现代显卡架构上. 这就是为什么当你玩游戏卡时, 多更新显卡驱动可能会有明显的效果.  
因为显卡驱动的复杂性, 应用程序开发人员必须处理不同显卡驱动之间的差异性, 例如不同厂商的显卡使用的着色器语言的语法不一致. 在过去的十年里, 除了显卡增加的新功能, 拥有强大图形硬件的移动设备也大量出现. 根据移动设备的大小以及电池容量不同, 其上的GPU架构也不同. 例如: [tiled rendering](https://en.wikipedia.org/wiki/Tiled_rendering), 通过给程序员提供更多对该功能的控制, 其性能也将大大提升. 另一项来源于老API的限制是其对多线程的支持度不够, 这会在CPU端导致性能瓶颈.  

福尔康是为现代图形架构设计的API. 它允许程序员使用一些啰嗦冗长的API来更清楚地指明他们的意图, 以此来减轻驱动的负担. 它支持多线程同时创建及提交命令. 它通过使用单独的编译器将glsl编译成标准字节码格式来解决不同显卡着色的差异问题. 最后, 它将图形和计算功能统一为一个API, 充分发挥现代显卡的通用处理能力.  

## 画一个三角形要准备些什么  

现在, 我们将大概说明在一个完整的福尔康程序中渲染一个三角形需要哪些步骤. 在之后的章节里会阐述这里涉及到的所有概念. 这里只能让你有个大概的认识.    

### 步骤1 - 实例(instance)和物理设备的选择

一个福尔康应用首先要通过实例设置好福尔康的API. 实例通过提供你的应用的信息以及你将使用的API拓展来创建. 在创建好了实例之后, 你可以查询支持福尔康的硬件并从中选择一个或者多个物理设备用于之后的操作. 你也可以通过查询一些更详细的信息, 例如VRAM大小或设备能力(device capabilities)来选择你想要的设备, 比如, 你可能更喜欢使用专用的图形显卡.  

### 步骤2 - 逻辑设备和队列族(queue families)

在选好了要用的硬件设备后, 你需要创建一个逻辑设备, 你可以通过它来具体指定你将用到的物理设备的特性, 例如多重视口渲染, 64位浮点数等等. 你还需要指定你将使用的队列族. 大多数使用福尔康执行的操作, 例如绘制命令和内存操作, 都是通过将其提交到命令队列后异步执行的. 命令队列从队列族中分配, 每个队列族都支持一种特定的操作集合. 例如, 对于图形显示, 计算和内存传输操作, 都分别有单独的队列族. 队列族的功能也可以作为选择物理设备时的一个区分因素. 支持福尔康的设备可能不提供任何图形显示功能, 不过目前支持福尔康的所有显卡通常都支持我们需要的队列功能.  

### 步骤3 - 窗口surface和交换链

你需要创建一个窗口来展示你所渲染的图像, 除非你只关心离屏渲染. 你可以使用你喜欢的API来创建窗口. 创建窗口的部分和福尔康无关.   

在渲染图像到窗口的过程中, 你需要另外两个组建: 窗口surface以及交换链. 福尔康本身不依赖于平台. surface是窗口上的跨平台的抽象.  

交换链是一系列渲染对象的集合. 它的作用在于确保我们正在渲染的图像和正在屏幕上显示的图像不同. 每当我们想绘制一帧图像, 我们必须要求交换链提供给我们一个要渲染的图像. 当绘制完一帧之后, 图像将返回交换链中, 用以在未来某个时刻显示到屏幕上. 渲染对象的数量和将渲染完成的图像显示到屏幕上的条件取决于交换链的显示模式. 常用的显示模式有双缓冲(垂直同步)以及三重缓冲. 我们将在之后详细介绍这些东西.  

有些平台允许你直接渲染到显示器上(display), 而不需要与任何窗口管理器交互. 你可以通过这种方式创建一个覆盖整个屏幕的surface, 然后通过这种方式实现一个自己的窗口管理器.  

### 步骤4 - 图像视图和帧缓冲

为了绘制从交换链中获取的图像, 我们必须把它包装成图像视图和帧缓冲. 图像视图是指使用的图像的某部分, 帧缓冲则是将用于颜色, 深度, 模板目标的图像视图. 因为在交换链中的图像各不相同, 我们必须为交换链中的每个图像分别创建一个图像视图和帧缓冲. 在渲染的时候, 选择正确的图像视图和帧缓冲.  


### 步骤5 - render passes

render passes 用来描述在渲染操作时使用的图像的类型, 以及它们是如何被使用的, 以及如何处理它们的内容. 在入门的三角形应用中, 我们告诉福尔康我们将使用单个图像作为颜色目标, 并且在绘制操作执行之前, 我们将其清理为某个纯色的背景. 虽然 render passes 只描述图像的类型, 不过帧缓冲实际上将特定图像绑定到这些插槽上.  

### 步骤6 - 图形管线

渲染管线描述了显卡的可配置状态, 例如视口尺寸和深度缓冲操作以及可编程阶段的着色器模块. 着色器模块是从着色器二进制代码中创建的. 驱动同时需要知道哪个渲染对象将在管线中被使用. 我们通过上面的 render pass 来指定.  

与现有的API相比, 福尔康最显著的特点是, 图形管线的所有可配置状态都需要手动进行设置. 这也意味着如果你想切换不同的着色器或者只是想修改顶点布局, 你都必须完整地重建整个图形管线对象. 只有一些基础的设置, 例如视口尺寸和清理颜色, 可以被动态修改. 所有的状态都要明确指定, 福尔康并不会为其提供默认值.  

这样的好处在于, 方便优化.  

### 步骤7 - 命令池和命令缓冲

正如之前提到的, 我们想在福尔康中进行的诸多操作, 例如绘制, 都需要提交到一个队列中. 在提交之前, 这些命令首先需要被记录到命令缓冲中. 命令缓冲从命令池中分配, 而命令池则与特定的队列族相互关联. 为了绘制一个简单的三角形, 我们需要在命令缓冲中记录下如下命令:  

* 启动 render pass
* 绑定图形管线
* 绘制3个顶点
* 结束 render pass

因为在帧缓冲中的图像取决于交换链给我们什么样的图像, 因此, 我们需要为每个可能的图像都记录一个命令缓冲, 然后再绘制时选择正确的命令缓冲. 另一个方法是, 再每一帧时重新记录命令缓冲区, 不过这个办法效率不高.  

### 步骤8 - 主循环

现在, 绘制命令已经被包装为了命令缓冲, 主循环的结构就很清晰了. 我们首先从交换链中获取一张图像, 然后为该图像选择合适的命令缓冲, 提交并执行命令. 最后, 我们将图像返回给交换链, 交换链再将图像展示到屏幕上.  
 
提交到队列的操作将被异步执行. 因此我们必须使用像信号量这样的同步对象来确保指令的执行顺序正确. 只有等到图像获取操作完成之后, 才能执行绘制操作, 否则我们会把之前渲染完要显示到屏幕上的图像再渲染一次. 而屏幕显示这个操作需要等待渲染操作完成, 因此我们使用第二个信号量, 该信号量将在渲染完成后发出信号.  

### 总结

本章对福尔康的快速浏览将会让你对未来如何绘制一个三角形有个基础的认识. 一个真实的应用将会包含更多步骤, 例如分配顶点缓冲, 创建uniform缓冲, 以及上传纹理图片, 这些都将在未来的章节中介绍, 不过现在还是简单些好吗因为福尔康的学习曲线过于陡峭. 注意, 我们将要写的三角形程序所使用的顶点将直接写在顶点着色器中, 而不是使用顶点缓冲区. 这是因为学习有关顶点缓冲区的管理的内容首先要对命令缓冲很熟悉.  

简单来说, 为了绘制第一个三角形, 我们需要:  

* 创建实例
* 选择支持的显卡
* 创建逻辑设备和队列
* 创建物理窗口, 逻辑窗口和交换链
* 将交换链图像包装为图像视图
* 创建render pass, 指定渲染对象和用法
* 为render pass 创建帧缓冲
* 设置图形管线
* 分配并记录命令缓冲
* 交换交换链
* 获取图像, 提交命令开始绘制, 绘制完成后将图像返回交换链并展示

## API concepts

This chapter will conclude with a short overview of how the Vulkan API is
structured at a lower level.

### Coding conventions

All of the Vulkan functions, enumerations and structs are defined in the
`vulkan.h` header, which is included in the [Vulkan SDK](https://lunarg.com/vulkan-sdk/)
developed by LunarG. We'll look into installing this SDK in the next chapter.

Functions have a lower case `vk` prefix, types like enumerations and structs
have a `Vk` prefix and enumeration values have a `VK_` prefix. The API heavily
uses structs to provide parameters to functions. For example, object creation
generally follows this pattern:

```c++
VkXXXCreateInfo createInfo = {};
createInfo.sType = VK_STRUCTURE_TYPE_XXX_CREATE_INFO;
createInfo.pNext = nullptr;
createInfo.foo = ...;
createInfo.bar = ...;

VkXXX object;
if (vkCreateXXX(&createInfo, nullptr, &object) != VK_SUCCESS) {
    std::cerr << "failed to create object" << std::endl;
    return false;
}
```

Many structures in Vulkan require you to explicitly specify the type of
structure in the `sType` member. The `pNext` member can point to an extension
structure and will always be `nullptr` in this tutorial. Functions that create
or destroy an object will have a VkAllocationCallbacks parameter that allows you
to use a custom allocator for driver memory, which will also be left `nullptr`
in this tutorial.

Almost all functions return a VkResult that is either `VK_SUCCESS` or an error
code. The specification describes which error codes each function can return and
what they mean.

### Validation layers

As mentioned earlier, Vulkan is designed for high performance and low driver
overhead. Therefore it will include very limited error checking and debugging
capabilities by default. The driver will often crash instead of returning an
error code if you do something wrong, or worse, it will appear to work on your
graphics card and completely fail on others.

Vulkan allows you to enable extensive checks through a feature known as
*validation layers*. Validation layers are pieces of code that can be inserted
between the API and the graphics driver to do things like running extra checks
on function parameters and tracking memory management problems. The nice thing
is that you can enable them during development and then completely disable them
when releasing your application for zero overhead. Anyone can write their own
validation layers, but the Vulkan SDK by LunarG provides a standard set of
validation layers that we'll be using in this tutorial. You also need to
register a callback function to receive debug messages from the layers.

Because Vulkan is so explicit about every operation and the validation layers
are so extensive, it can actually be a lot easier to find out why your screen is
black compared to OpenGL and Direct3D!

There's only one more step before we'll start writing code and that's [setting
up the development environment](!Development_environment).

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

Note the `KHR` postfix, which
means that these objects are part of a Vulkan extension. The Vulkan API itself
is completely platform agnostic, which is why we need to use the standardized
WSI (Window System Interface) extension to interact with the window manager. The
surface is a cross-platform abstraction over windows to render to and is
generally instantiated by providing a reference to the native window handle, for
example `HWND` on Windows. Luckily, the GLFW library has a built-in function to
deal with the platform specific details of this.
在渲染图像到窗口的过程中, 你需要另外两个组建: 窗口surface以及交换链. 福尔康本身不依赖于平台. surface是窗口上的跨平台的抽象

The swap chain is a collection of render targets. Its basic purpose is to ensure
that the image that we're currently rendering to is different from the one that
is currently on the screen. This is important to make sure that only complete
images are shown. Every time we want to draw a frame we have to ask the swap
chain to provide us with an image to render to. When we've finished drawing a
frame, the image is returned to the swap chain for it to be presented to the
screen at some point. The number of render targets and conditions for presenting
finished images to the screen depends on the present mode. Common present modes
are  double buffering (vsync) and triple buffering. We'll look into these in the
swap chain creation chapter.

Some platforms allow you to render directly to a display without interacting with any window manager through the `VK_KHR_display` and `VK_KHR_display_swapchain` extensions. These allow you to create a surface that represents the entire screen and could be used to implement your own window manager, for example.

### Step 4 - Image views and framebuffers

To draw to an image acquired from the swap chain, we have to wrap it into a
VkImageView and VkFramebuffer. An image view references a specific part of an
image to be used, and a framebuffer references image views that are to be used
for color, depth and stencil targets. Because there could be many different
images in the swap chain, we'll preemptively create an image view and
framebuffer for each of them and select the right one at draw time.

### Step 5 - Render passes

Render passes in Vulkan describe the type of images that are used during
rendering operations, how they will be used, and how their contents should be
treated. In our initial triangle rendering application, we'll tell Vulkan that
we will use a single image as color target and that we want it to be cleared
to a solid color right before the drawing operation. Whereas a render pass only
describes the type of images, a VkFramebuffer actually binds specific images to
these slots.

### Step 6 - Graphics pipeline

The graphics pipeline in Vulkan is set up by creating a VkPipeline object. It
describes the configurable state of the graphics card, like the viewport size
and depth buffer operation and the programmable state using VkShaderModule
objects. The VkShaderModule objects are created from shader byte code. The
driver also needs to know which render targets will be used in the pipeline,
which we specify by referencing the render pass.

One of the most distinctive features of Vulkan compared to existing APIs, is
that almost all configuration of the graphics pipeline needs to be set in advance.
That means that if you want to switch to a different shader or slightly
change your vertex layout, then you need to entirely recreate the graphics
pipeline. That means that you will have to create many VkPipeline objects in
advance for all the different combinations you need for your rendering
operations. Only some basic configuration, like viewport size and clear color,
can be changed dynamically. All of the state also needs to be described
explicitly, there is no default color blend state, for example.

The good news is that because you're doing the equivalent of ahead-of-time
compilation versus just-in-time compilation, there are more optimization
opportunities for the driver and runtime performance is more predictable,
because large state changes like switching to a different graphics pipeline are
made very explicit.

### Step 7 - Command pools and command buffers

As mentioned earlier, many of the operations in Vulkan that we want to execute,
like drawing operations, need to be submitted to a queue. These operations first
need to be recorded into a VkCommandBuffer before they can be submitted. These
command buffers are allocated from a `VkCommandPool` that is associated with a
specific queue family. To draw a simple triangle, we need to record a command
buffer with the following operations:

* Begin the render pass
* Bind the graphics pipeline
* Draw 3 vertices
* End the render pass

Because the image in the framebuffer depends on which specific image the swap
chain will give us, we need to record a command buffer for each possible image
and select the right one at draw time. The alternative would be to record the
command buffer again every frame, which is not as efficient.

### Step 8 - Main loop

Now that the drawing commands have been wrapped into a command buffer, the main
loop is quite straightforward. We first acquire an image from the swap chain
with vkAcquireNextImageKHR. We can then select the appropriate command buffer
for that image and execute it with vkQueueSubmit. Finally, we return the image
to the swap chain for presentation to the screen with vkQueuePresentKHR.

Operations that are submitted to queues are executed asynchronously. Therefore
we have to use synchronization objects like semaphores to ensure a correct
order of execution. Execution of the draw command buffer must be set up to wait
on image acquisition to finish, otherwise it may occur that we start rendering
to an image that is still being read for presentation on the screen. The
vkQueuePresentKHR call in turn needs to wait for rendering to be finished, for
which we'll use a second semaphore that is signaled after rendering completes.

### Summary

This whirlwind tour should give you a basic understanding of the work ahead for
drawing the first triangle. A real-world program contains more steps, like
allocating vertex buffers, creating uniform buffers and uploading texture images
that will be covered in subsequent chapters, but we'll start simple because
Vulkan has enough of a steep learning curve as it is. Note that we'll cheat a
bit by initially embedding the vertex coordinates in the vertex shader instead
of using a vertex buffer. That's because managing vertex buffers requires some
familiarity with command buffers first.

So in short, to draw the first triangle we need to:

* Create a VkInstance
* Select a supported graphics card (VkPhysicalDevice)
* Create a VkDevice and VkQueue for drawing and presentation
* Create a window, window surface and swap chain
* Wrap the swap chain images into VkImageView
* Create a render pass that specifies the render targets and usage
* Create framebuffers for the render pass
* Set up the graphics pipeline
* Allocate and record a command buffer with the draw commands for every possible
swap chain image
* Draw frames by acquiring images, submitting the right draw command buffer and
returning the images back to the swap chain

It's a lot of steps, but the purpose of each individual step will be made very
simple and clear in the upcoming chapters. If you're confused about the relation
of a single step compared to the whole program, you should refer back to this
chapter.

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

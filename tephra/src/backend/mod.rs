pub mod vulkan;
use buffer::BufferApi;
use shader::ShaderApi;
//use renderpass::RenderpassApi;
//use pipeline::PipelineApi;
use descriptor::{DescriptorApi, LayoutApi};
use image::ImageApi;
use render::RenderApi;
use swapchain::SwapchainApi;
use render::ComputeApi;

pub trait BackendApi
where
    Self: Copy + Clone + Sized + 'static,
{
    type Context: Clone;
    type Shader: ShaderApi;
    type Buffer;
    type Render: RenderApi;
    type Compute: ComputeApi;
    type Image;
    type Swapchain: SwapchainApi;
    type Descriptor;
    type Layout: LayoutApi;
}

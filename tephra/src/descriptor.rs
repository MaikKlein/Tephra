use crate::{
    buffer::BufferHandle,
    commandbuffer::{Descriptor, ShaderView, ShaderViews},
    context::Context,
};

use std::collections::HashMap;

crate::new_typed_handle!(DescriptorHandle);

pub trait CreateDescriptor {}

pub trait CreatePool {
    fn create_pool(
        &self,
        alloc_size: u32,
        data: &[ShaderView],
        sizes: DescriptorSizes,
    ) -> NativePool;
}

pub trait PoolApi {
    fn create_descriptor(&self, count: u32) -> Vec<DescriptorHandle>;
}

pub struct NativePool {
    pub inner: Box<dyn PoolApi>,
}

pub struct LinearPoolAllocator {
    ctx: Context,
    block_size: usize,
    pools: Vec<NativePool>,
    descriptors: Vec<DescriptorHandle>,
    // Infos
    views: ShaderViews,
    sizes: DescriptorSizes,
    current_allocations: usize,
}

impl LinearPoolAllocator {
    pub fn new(ctx: &Context, views: ShaderViews) -> Self {
        let sizes = DescriptorSizes::from_views(&views);
        LinearPoolAllocator {
            ctx: ctx.clone(),
            block_size: 50,
            pools: Vec::new(),
            descriptors: Vec::new(),
            views,
            sizes,
            current_allocations: 0,
        }
    }

    pub fn create_descriptor(&mut self) -> DescriptorHandle {
        let allocator_index = self.current_allocations / self.block_size;
        // If we don't have enough space, we need to allocate a new pool
        if allocator_index >= self.pools.len() {
            self.allocate_additional_pool();
        }
        let handle = self.descriptors[self.current_allocations];
        self.current_allocations += 1;
        handle
    }
    pub fn allocate_additional_pool(&mut self) {
        let pool = self
            .ctx
            .create_pool(self.block_size as u32, &self.views, self.sizes);
        let descriptors = pool.inner.create_descriptor(self.block_size as u32);
        self.descriptors.extend(descriptors);
        self.pools.push(pool);
    }

    pub fn reset(&mut self) {
        for _pool in &mut self.pools {
            self.current_allocations = 0;
        }
    }
}

pub struct Pool {
    ctx: Context,
    allocators: HashMap<ShaderViews, LinearPoolAllocator>,
}

impl Pool {
    pub fn new(ctx: &Context) -> Self {
        Pool {
            ctx: ctx.clone(),
            allocators: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, data: &Descriptor) -> DescriptorHandle {
        let ctx = self.ctx.clone();
        let allocator = self
            .allocators
            .entry(data.views.clone())
            .or_insert_with(|| LinearPoolAllocator::new(&ctx, data.views.clone()));
        let handle = allocator.create_descriptor();
        ctx.write(handle, &data);
        handle
    }

    pub fn reset(&mut self) {
        for allocator in self.allocators.values_mut() {
            allocator.reset();
        }
    }
}

pub trait DescriptorApi {
    fn write(&self, handle: DescriptorHandle, data: &Descriptor);
}

#[derive(Debug, Copy, Clone)]
pub struct DescriptorSizes {
    pub buffer: u32,
    pub storage: u32,
    pub images: u32,
}

impl DescriptorSizes {
    pub fn from_views(views: &[ShaderView]) -> Self {
        let sizes = DescriptorSizes {
            buffer: 0,
            storage: 0,
            images: 0,
        };
        views.iter().fold(sizes, |mut acc, elem| {
            match elem.ty {
                DescriptorType::Uniform => acc.buffer += 1,
                DescriptorType::Storage => acc.storage += 1,
            }
            acc
        })
    }
}

pub trait DescriptorInfo
where
    Self: 'static,
{
    fn descriptor_data(&self) -> Vec<Binding<DescriptorResource>>;
    fn layout() -> Vec<Binding<DescriptorType>>;
}
impl DescriptorInfo for () {
    fn descriptor_data(&self) -> Vec<Binding<DescriptorResource>> {
        Vec::new()
    }
    fn layout() -> Vec<Binding<DescriptorType>> {
        Vec::new()
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum DescriptorType {
    Uniform,
    Storage,
}
pub enum DescriptorResource {
    Uniform(BufferHandle),
    Storage(BufferHandle),
}
#[derive(Debug)]
pub struct Binding<T> {
    pub binding: u32,
    pub data: T,
}

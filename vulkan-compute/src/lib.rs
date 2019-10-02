
// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

// This example demonstrates how to use the compute capabilities of Vulkan.
//
// While graphics cards have traditionally been used for graphical operations, over time they have
// been more or more used for general-purpose operations as well. This is called "General-Purpose
// GPU", or *GPGPU*. This is what this example demonstrates.

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::GpuFuture;
use vulkano::sync;

use std::sync::Arc;

pub mod prelude {
    pub const EPSILON: f64 = 0.0005;
    pub use super::{MarchInstruction, MarchResult};    
}

#[derive(Debug, Clone, Copy)]
pub struct MarchResult {
    pub distance: f32,
    pub normal: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct MarchInstruction {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Default for MarchInstruction {
    fn default() -> Self {
        MarchInstruction {
            origin: [0.0; 3],
            direction: [0.0; 3],
        }
    }
}

impl From<MarchInstruction> for MarchResult {
    fn from(instr: MarchInstruction) -> Self {
        MarchResult {
            distance: instr.origin[0],
            normal: instr.direction,
        }
    }
}


pub fn compute(data: &Vec<MarchInstruction>) -> Vec<MarchResult> {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();

    // Choose which physical device to use.
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
    // that supports compute operations.
    let queue_family = physical.queue_families().find(|&q| q.supports_compute()).unwrap();

    // Now initializing the device.
    let (device, mut queues) = Device::new(physical, physical.supported_features(),
        &DeviceExtensions::none(), [(queue_family, 0.5)].iter().cloned()).unwrap();

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
    // example we use only one queue, so we just retrieve the first and only element of the
    // iterator and throw it away.
    let queue = queues.next().unwrap();

    println!("Device initialized");


    // We need to create the compute pipeline that describes our operation.
        mod cs {
            vulkano_shaders::shader!{
                ty: "compute",
                src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct InputData {
	vec3 origin;
	vec3 dir;
};

layout(set = 0, binding = 0) buffer Data {
	InputData data[];
} data;


float sphere(vec3 p, float radius) {
    return length(p) - radius;
}

float scene(vec3 p) {
    return sphere(p, 1.5);
}


float EPSILON = 0.0001;

vec3 gradient(vec3 pos) {
	return normalize(vec3(
		scene(pos + vec3(EPSILON, 0.0, 0.0)) - scene(pos - vec3(EPSILON, 0.0, 0.0)),
		scene(pos + vec3(0.0, EPSILON, 0.0)) - scene(pos - vec3(0.0, EPSILON, 0.0)),
		scene(pos + vec3(0.0, 0.0, EPSILON)) - scene(pos - vec3(0.0, 0.0, EPSILON))
	));
}


void main() {
    int MAX_STEPS = 50;

    uint idx = gl_GlobalInvocationID.x;

    
    vec3 origin = data.data[idx].origin;
    vec3 direction = data.data[idx].dir;


    float depth = 0.0001;
    for (int i = 0; i < MAX_STEPS; i++) {
        if (depth < 0.001 || depth > 10000.0) { continue; }
        vec3 frag_pos = origin + direction * depth;
        
        float dist = scene(frag_pos);

        depth += dist;
    }

    data.data[idx].origin.x = depth;
    data.data[idx].dir = gradient(origin + direction * depth);
}"
            }
        }
    let pipeline = Arc::new({
        let shader = cs::Shader::load(device.clone()).unwrap();
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap()
    });

    // We start by creating the buffer that will store the data.
    let data_buffer = {
        // Iterator that produces the data.
        let data_iter = data.iter().map(|i| cs::ty::InputData {
				dir: i.direction,
				origin: i.origin,
				_dummy0: Default::default(),
				_dummy1: Default::default(),
			}
		);
        // Builds the buffer and fills it with this iterator.
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter).unwrap()
    };

    let set = Arc::new(PersistentDescriptorSet::start(pipeline.clone(), 0)
        .add_buffer(data_buffer.clone()).unwrap()
        .build().unwrap()
    );

    dbg!(data.len() / 64);

    let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
        .dispatch([data.len() as u32 / 64, 1, 1], pipeline.clone(), set.clone(), ()).unwrap()
        .build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer).unwrap()

        // This line instructs the GPU to signal a *fence* once the command buffer has finished
        // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
        // reached a certain point.
        // We need to signal a fence here because below we want to block the CPU until the GPU has
        // reached that point in the execution.
        .then_signal_fence_and_flush().unwrap();

    // Blocks execution until the GPU has finished the operation. This method only exists on the
    // future that corresponds to a signalled fence. In other words, this method wouldn't be
    // available if we didn't call `.then_signal_fence_and_flush()` earlier.
    // The `None` parameter is an optional timeout.
    future.wait(None).unwrap();

    // Now that the GPU is done, the content of the buffer should have been modified. Let's
    // check it out.
    // The call to `read()` would return an error if the buffer was still in use by the GPU.
    let data_buffer_content = data_buffer.read().unwrap();

    data_buffer_content.iter().map(|data| {
        MarchResult {
            distance: data.origin[0],
            normal: data.dir,
        }
    }).collect()
}

use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph,
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
    },
};

use crate::image::SmoothLifeImage;
// use crate::utils::SmoothLifeImage;
use crate::SIM_SIZE;
use crate::WORKGROUP_SIZE;

#[derive(Resource)]
pub struct SmoothLifePipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for SmoothLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("smooth life bind group layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/smooth_life.wgsl");

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("smooth life init pipeline")),
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("smooth life update pipeline")),
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        SmoothLifePipeline {
            init_pipeline,
            update_pipeline,
            texture_bind_group_layout,
        }
    }
}

#[derive(Resource)]
struct SmoothLifeImageBindGroup(pub BindGroup);

pub fn queue_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<SmoothLifePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    smooth_life_image: Res<SmoothLifeImage>,
) {
    let view = &gpu_images[&smooth_life_image.0];

    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("smooth life bind group"),
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });
    commands.insert_resource(SmoothLifeImageBindGroup(bind_group));
}

pub enum SmoothLifeState {
    Loading,
    Init,
    Update,
}

pub struct SmoothLifeNode {
    state: SmoothLifeState,
}

impl Default for SmoothLifeNode {
    fn default() -> Self {
        Self {
            state: SmoothLifeState::Loading,
        }
    }
}

impl render_graph::Node for SmoothLifeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<SmoothLifePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            SmoothLifeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = SmoothLifeState::Init;
                }
            }
            SmoothLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = SmoothLifeState::Update;
                }
            }
            SmoothLifeState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_cx: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<SmoothLifeImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<SmoothLifePipeline>();

        let mut pass = render_cx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        match self.state {
            SmoothLifeState::Loading => {}
            SmoothLifeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(
                    SIM_SIZE.0 / WORKGROUP_SIZE,
                    SIM_SIZE.1 / WORKGROUP_SIZE,
                    1,
                );
            }
            SmoothLifeState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(
                    SIM_SIZE.0 / WORKGROUP_SIZE,
                    SIM_SIZE.1 / WORKGROUP_SIZE,
                    1,
                );
            }
        }

        Ok(())
    }
}

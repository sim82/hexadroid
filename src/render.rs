use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAsset,
        render_resource::{BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor},
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPipeline, Material2dPlugin},
};

#[derive(Clone, TypeUuid)]
#[uuid = "2f58e6f2-9680-4311-ab7b-80b948d6ba18"]
pub struct MyMaterial {}

pub struct MyMaterialGpu {
    bind_group: BindGroup,
}
impl Material2d for MyMaterial {
    fn bind_group(material: &MyMaterialGpu) -> &bevy::render::render_resource::BindGroup {
        &material.bind_group
    }

    fn bind_group_layout(
        render_device: &bevy::render::renderer::RenderDevice,
    ) -> bevy::render::render_resource::BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        })
    }
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        asset_server.watch_for_changes().unwrap();
        Some(asset_server.load("my_material.wgsl"))
    }
}

impl RenderAsset for MyMaterial {
    type ExtractedAsset = MyMaterial;

    type PreparedAsset = MyMaterialGpu;

    type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<MyMaterial>>);

    fn extract_asset(&self) -> MyMaterial {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, pipeline): &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.material2d_layout,
            entries: &[],
        });
        Ok(MyMaterialGpu { bind_group })
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<MyMaterial>::default());
    }
}

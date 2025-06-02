use avian2d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[allow(unused)]
pub fn collision_started_events_both_ways<'a>(
    reader: &'a mut EventReader<CollisionStarted>,
) -> impl 'a + Iterator<Item = (Entity, Entity)> {
    reader
        .read()
        .flat_map(|CollisionStarted(e1, e2)| [(*e1, *e2), (*e2, *e1)])
}

#[derive(SystemParam)]
pub struct CachedPbrMaker<'w, 's> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    mesh_and_material: Local<'s, Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
}

impl CachedPbrMaker<'_, '_> {
    pub fn make_pbr_with(
        &mut self,
        mesh: impl FnOnce() -> Mesh,
        material: impl FnOnce() -> StandardMaterial,
    ) -> (Mesh3d, MeshMaterial3d<StandardMaterial>) {
        let (mesh, material) = self
            .mesh_and_material
            .get_or_insert_with(|| (self.meshes.add(mesh()), self.materials.add(material())))
            .clone();
        (Mesh3d(mesh), MeshMaterial3d(material))
    }
}

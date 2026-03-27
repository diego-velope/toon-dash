//! Load glTF 2.0 / GLB into Macroquad [`Mesh`] for rendering with [`draw_mesh`].
//!
//! Kenney GLBs often reference an external `Textures/colormap.png`. [`gltf::import_slice`]
//! cannot resolve that URI. We parse the GLB with [`gltf::Gltf::from_slice`] and load only
//! **buffers** (geometry in the BIN chunk), skipping external images — vertex colors / PBR
//! factors still work for solid shading.
//!
//! Kenney kit meshes use one **texture atlas** (`Textures/colormap.png`) for albedo; without it,
//! materials read as white. Pass the loaded atlas as [`Texture2D`] when building the mesh.
//!
//! Game: Y-up, +Z forward — use [`draw_mesh_at_rot`] to orient Kenney assets.

use glam::Quat;
use gltf::mesh::Mode;
use gltf::Document;
use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::*;

/// Load all triangle primitives from every mesh (Kenney characters often use several parts).
///
/// `atlas` should be the kit `colormap.png` when materials use `baseColorTexture` (typical Kenney).
pub fn mesh_from_glb_bytes(bytes: &[u8], atlas: Option<Texture2D>) -> Result<Mesh, String> {
    let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| format!("gltf parse: {e}"))?;
    let document = gltf.document;
    let buffers = gltf::import_buffers(&document, None, gltf.blob)
        .map_err(|e| format!("gltf buffers: {e}"))?;
    mesh_from_document(&document, &buffers, atlas)
}

pub fn mesh_from_document(
    document: &Document,
    buffers: &[gltf::buffer::Data],
    atlas: Option<Texture2D>,
) -> Result<Mesh, String> {
    let mut all_vertices: Vec<Vertex> = Vec::new();
    let mut all_indices: Vec<u16> = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            append_primitive(&primitive, buffers, &mut all_vertices, &mut all_indices)?;
        }
    }

    if all_vertices.is_empty() {
        return Err("no triangle geometry in glTF".into());
    }

    Ok(Mesh {
        vertices: all_vertices,
        indices: all_indices,
        texture: atlas,
    })
}

fn append_primitive(
    primitive: &gltf::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
    all_vertices: &mut Vec<Vertex>,
    all_indices: &mut Vec<u16>,
) -> Result<(), String> {
    if primitive.mode() != Mode::Triangles {
        return Ok(());
    }

    let material = primitive.material();
    let pbr = material.pbr_metallic_roughness();
    let f = pbr.base_color_factor();

    let reader =
        primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| data.0.as_slice()));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or_else(|| "missing POSITION attribute".to_string())?
        .collect();

    if positions.is_empty() {
        return Ok(());
    }

    let tex_coords: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|tc| tc.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

    let colors: Vec<[f32; 4]> = reader
        .read_colors(0)
        .map(|c| c.into_rgba_f32().collect())
        .unwrap_or_else(|| vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]);

    let base = all_vertices.len() as u32;

    for i in 0..positions.len() {
        let vc = colors[i];
        let vertex_color = Color::from_rgba(
            (vc[0] * f[0] * 255.0) as u8,
            (vc[1] * f[1] * 255.0) as u8,
            (vc[2] * f[2] * 255.0) as u8,
            (vc[3] * f[3] * 255.0) as u8,
        );

        all_vertices.push(Vertex::new2(
            vec3(positions[i][0], positions[i][1], positions[i][2]),
            vec2(tex_coords[i][0], tex_coords[i][1]),
            vertex_color,
        ));
    }

    match reader.read_indices() {
        Some(iter) => {
            for idx in iter.into_u32() {
                let global = base + idx;
                all_indices.push(
                    u16::try_from(global)
                        .map_err(|_| "mesh index overflow — model too complex for u16 indices")?,
                );
            }
        }
        None => {
            if positions.len() > usize::from(u16::MAX) {
                return Err("unindexed mesh too large".into());
            }
            for i in 0..positions.len() as u32 {
                all_indices.push((base + i) as u16);
            }
        }
    }

    Ok(())
}

/// Draw a mesh with translation and uniform scale (world Y-up, +Z forward).
pub fn draw_mesh_at(mesh: &Mesh, position: Vec3, scale: f32) {
    draw_mesh_at_rot(mesh, position, scale, Quat::IDENTITY);
}

/// Same as [`draw_mesh_at`] but applies a rotation in model space first (e.g. fix Kenney export facing).
pub fn draw_mesh_at_rot(mesh: &Mesh, position: Vec3, scale: f32, rot: Quat) {
    let mut verts = Vec::with_capacity(mesh.vertices.len());
    for v in &mesh.vertices {
        let mut nv = *v;
        nv.position = rot * (v.position * scale) + position;
        verts.push(nv);
    }
    let m = Mesh {
        vertices: verts,
        indices: mesh.indices.clone(),
        texture: mesh.texture.clone(),
    };
    draw_mesh(&m);
}

/// Draw a mesh with translation, non-uniform scale (Vec3), and rotation.
pub fn draw_mesh_at_transform(mesh: &Mesh, position: Vec3, scale: Vec3, rot: Quat) {
    let mut verts = Vec::with_capacity(mesh.vertices.len());
    for v in &mesh.vertices {
        let mut nv = *v;
        // Multiply element-wise for non-uniform scale before rotating
        nv.position = rot * (v.position * scale) + position;
        verts.push(nv);
    }
    let m = Mesh {
        vertices: verts,
        indices: mesh.indices.clone(),
        texture: mesh.texture.clone(),
    };
    draw_mesh(&m);
}

#[cfg(test)]
mod tests {
    use super::mesh_from_glb_bytes;
    use std::path::PathBuf;

    fn asset(p: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(p)
    }

    #[test]
    fn kenney_glbs_parse_without_external_texture_import() {
        for rel in [
            "models/road-straight.glb",
            "models/character-oodi.glb",
            "models/coin-gold.glb",
            "models/fence-low-straight.glb",
            "models/spike-block.glb",
            "models/poles.glb",
        ] {
            let bytes = std::fs::read(asset(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"));
            mesh_from_glb_bytes(&bytes, None).unwrap_or_else(|e| panic!("mesh {rel}: {e}"));
        }
    }
}

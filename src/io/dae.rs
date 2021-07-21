use collada::document::LambertDiffuse;
use collada::document::MaterialEffect;
use collada::TVertex;
use collada::Vertex;

use crate::definition::*;
use crate::io::*;
use std::path::Path;

// type VertexIndex = usize;
// type TextureIndex = usize;
// type NormalIndex = usize;
// type DaeVTNIndex = (VertexIndex, Option<TextureIndex>, Option<NormalIndex>);

impl Loaded {
    ///
    /// Deserialize a loaded .dae file resource
    pub fn dae<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(Vec<CPUMesh>, Vec<CPUMaterial>), IOError> {
        let dae_bytes = self.remove_bytes(path.as_ref())?;
        let dae =
            collada::document::ColladaDocument::from_str(&*String::from_utf8(dae_bytes).unwrap())
                .unwrap();
        let p = path.as_ref().parent().unwrap();
        let obj_set = dae.get_obj_set().unwrap();
        // let mat_lib = obj_set.material_library; // apparently no materials?

        // Parse materials
        let material_to_effect = dae.get_material_to_effect();
        println!("material to effect: {:?}", material_to_effect);
        let effect_library = dae.get_effect_library();
        let images = dae.get_images();
        println!("images: {:?}", images);

        let mut cpu_materials = Vec::new();

        for (k, v) in material_to_effect {
            let material_name = &v[..];
            if let Some(effect) = effect_library.get(material_name) {
                if let MaterialEffect::Lambert(lambert) = effect.clone() {
                    let mut color;
                    let texture_name = match lambert.diffuse {
                        LambertDiffuse::Texture(texture) => {
                            color = lambert.emission;
                            Some(texture)
                        }
                        LambertDiffuse::Color(lambert_color) => {
                            color = lambert_color;
                            None
                        }
                    };

                    let texture = match texture_name {
                        Some(sampler) => {
                            let texture_id = sampler.replace("-sampler", "");
                            let image = if let Some(filename) = images.get(&texture_id[..]) {
                                let path = p.join(filename).to_str().unwrap().to_owned();
                                let image = self.image(&path)?;
                                Some(image)
                            } else {
                                None
                            };
                            image
                        }
                        _ => None,
                    };
                    println!("color: {:?}", color);

                    let material = CPUMaterial {
                        name: k.to_string(),
                        // color: Some((color[0], color[1], color[2], color[3])),
                        // color: None,
                        color: Some((0.5, 0.5, 0.5, 1.)),
                        color_texture: texture,
                        ..Default::default()
                    };
                    cpu_materials.push(material);
                };
            };
        }

        // Parse meshes
        let mut cpu_meshes = Vec::new();
        // println!("material: {:?}",mat_lib);
        for object in obj_set.objects.into_iter() {
            // Objects consisting of several meshes with different materials
            for geo in object.geometry.iter() {
                let mut positions = Vec::new();
                let mut normals = Vec::new();
                let mut uvs = Vec::new();
                let mut indices = Vec::new();

                // let mut map: HashMap<usize, usize> = HashMap::new();


                for shape in &geo.mesh[..] {
                    match shape {
                        collada::PrimitiveElement::Triangles(tris) => {
                            let tris = tris.clone();
                            tris.vertices.into_iter().enumerate().for_each(|(i , v)| {
                                // let mut index: Vec<u32> = vec![v.0 as u32,v.1 as u32,v.2 as u32];
                                let i = i as u32 * 3;
                                let mut index = vec![i, i + 1, i + 2];

                                indices.append(&mut index);

                                let v_0 = object.vertices[v.0];
                                let v_1 = object.vertices[v.1];
                                let v_2 = object.vertices[v.2];

                                let mut push_vert = |v: Vertex| {
                                    let mut v_vec = vec![v.x as f32, v.y as f32, v.z as f32];
                                    positions.append(&mut v_vec);
                                };
                                push_vert(v_0);
                                push_vert(v_1);
                                push_vert(v_2);
                            });

                            if let Some(tex_verts) = tris.tex_vertices {
                                tex_verts.into_iter().for_each(|v| {
                                    let uv_0 = object.tex_vertices[v.0];
                                    let uv_1 = object.tex_vertices[v.1];
                                    let uv_2 = object.tex_vertices[v.2];

                                    let mut push_tex_vert = |v: TVertex| {
                                        let mut uv_vec = vec![v.x as f32, v.y as f32];
                                        uvs.append(&mut uv_vec);
                                    };
                                    push_tex_vert(uv_0);
                                    push_tex_vert(uv_1);
                                    push_tex_vert(uv_2);
                                });
                            }
                            if let Some(norm_verts) = tris.normals {
                                norm_verts.into_iter().for_each(|v| {
                                    let n_0 = object.normals[v.0];
                                    let n_1 = object.normals[v.1];
                                    let n_2 = object.normals[v.2];

                                    let mut push_tex_vert = |v: Vertex| {
                                        let mut n_vec = vec![v.x as f32, v.y as f32, v.z as f32];
                                        normals.append(&mut n_vec);
                                    };
                                    push_tex_vert(n_0);
                                    push_tex_vert(n_1);
                                    push_tex_vert(n_2);
                                });
                            }
                        }
                        _ => {}
                    }
                }
                println!("normals: {:?}", normals);
                println!("normals len: {:?}", normals.len());
                println!("uvs len: {:?}", uvs.len());
                println!("uvs: {:?}", uvs);
                cpu_meshes.push(CPUMesh {
                    name: object.name.to_string(),
                    material_name: None,
                    positions,
                    indices: Some(Indices::U32(indices)),
                    normals: Some(normals),
                    uvs: Some(uvs),
                    colors: None,
                });
            }
        }

        Ok((cpu_meshes, cpu_materials))
    }
}

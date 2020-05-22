use super::*;
use nalgebra::Rotation3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct CameraDesc {
    from: Point,
    at: Point,
    up: Vector,
    dist: f64,
    vfov: f64,
    aperture: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum TextureDesc {
    Solid { color: Color },
    Checker { texture1: String, texture2: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum MaterialDesc {
    Lambertian { texture: String },
    Metal { color: Color, roughness: f64 },
    Dielectric { ior: f64 },
    Emission { texture: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum GeomDesc {
    Sphere {
        material: String,
        center: Point,
        radius: f64,
    },
    Rect {
        material: String,
        axis: RectAxis,
        start: (f64, f64),
        end: (f64, f64),
        k: f64,
    },
    Block {
        material: String,
        start: Point,
        end: Point,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ObjectDesc {
    #[serde(flatten)]
    geometry: GeomDesc,
    rotate: Option<Vector>,
    translate: Option<Vector>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SceneDesc {
    camera: CameraDesc,
    textures: HashMap<String, TextureDesc>,
    materials: HashMap<String, MaterialDesc>,
    objects: Vec<ObjectDesc>,
}

fn transform(desc: &SceneDesc) -> Option<Scene> {
    let mut textures: HashMap<&str, Arc<Texture>> = HashMap::new();
    let mut materials: HashMap<&str, Arc<Material>> = HashMap::new();
    let mut result: Vec<Box<dyn Hittable>> = vec![];

    for (name, tex) in &desc.textures {
        let texture = match tex {
            &TextureDesc::Solid { color } => Texture::Solid(color),
            TextureDesc::Checker { texture1, texture2 } => {
                let t1 = textures.get(texture1.as_str())?.clone();
                let t2 = textures.get(texture2.as_str())?.clone();
                Texture::Checker(t1, t2)
            }
        };
        textures.insert(name, Arc::new(texture));
    }

    for (name, mat) in &desc.materials {
        let material = match mat {
            MaterialDesc::Lambertian { texture } => {
                let t = textures.get(texture.as_str())?.clone();
                Material::Lambertian(t)
            }
            MaterialDesc::Metal { color, roughness } => {
                Material::Metal(*color, *roughness)
            }
            MaterialDesc::Dielectric { ior } => Material::Dielectric(*ior),
            MaterialDesc::Emission { texture } => {
                let t = textures.get(texture.as_str())?.clone();
                Material::Emission(t)
            }
        };
        materials.insert(name, Arc::new(material));
    }

    for obj in &desc.objects {
        let mut hittable: Box<dyn Hittable> = match &obj.geometry {
            GeomDesc::Sphere {
                material,
                center,
                radius,
            } => {
                let m = materials.get(material.as_str())?.clone();
                Sphere::new(m, *center, *radius)
            }
            GeomDesc::Rect {
                material,
                axis,
                start,
                end,
                k,
            } => {
                let m = materials.get(material.as_str())?.clone();
                Rect::new(m, *axis, *start, *end, *k)
            }
            GeomDesc::Block {
                material,
                start,
                end,
            } => {
                let m = materials.get(material.as_str())?.clone();
                Block::new(m, *start, *end)
            }
        };

        if let Some(v) = &obj.rotate {
            hittable =
                Rotate::new(hittable, Rotation3::new(v.map(f64::to_radians)));
        }

        if let Some(v) = &obj.translate {
            hittable = Translate::new(hittable, *v);
        }

        result.push(hittable);
    }

    Some(Scene::from_objects(Color::zeros(), result))
}

pub(super) fn parse<P: AsRef<Path>>(
    config: &Config,
    path: P,
) -> (Scene, Camera) {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Read;
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut string = String::new();
    reader.read_to_string(&mut string).unwrap();
    let desc: SceneDesc = toml::from_str(&string).unwrap();

    let aspect_ratio: f64 = config.width as f64 / config.height as f64;
    let from = desc.camera.from;
    let at = desc.camera.at;
    let up = desc.camera.up;
    let dist = desc.camera.dist;
    let vfov = desc.camera.vfov;
    let aperture = desc.camera.aperture;
    let camera = Camera::new(from, at, up, vfov, aspect_ratio, aperture, dist);

    (transform(&desc).unwrap(), camera)
}

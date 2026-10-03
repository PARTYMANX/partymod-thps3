//! Xbox-style, bind-pose vertex scaling for create-a-skater weight/body parts.

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use partymod_common::{logger::LogLevel, patch};

use crate::logger;

type Vec3 = [f32; 3];

// The PC skin renderer reads a separate bind-pose buffer. RenderWare clones
// share that buffer, so never write into it or give Rust allocations to RW.
static SKIN_VERTICES: LazyLock<Mutex<HashMap<usize, Arc<[Vec3]>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

unsafe fn get_skin(atomic: *mut ()) -> *mut () {
    let function: unsafe extern "C" fn(*mut ()) -> *mut () =
        unsafe { std::mem::transmute(0x00575170usize) };
    unsafe { function(atomic) }
}

unsafe extern "C" fn render_atomic(atomic: *mut ()) -> *mut () {
    unsafe {
        let skin = get_skin(atomic);
        let vertices = SKIN_VERTICES.lock().unwrap().get(&(skin as usize)).cloned();
        let render: unsafe extern "C" fn(*mut ()) -> *mut () = std::mem::transmute(0x00577590usize);
        let Some(vertices) = vertices else {
            return render(atomic);
        };
        // Rendering is synchronous on the game thread. Restore the original
        // pointer before returning so cloning/destruction retain RW ownership.
        let slot = skin.byte_add(0x24).cast::<*const Vec3>();
        let original = *slot;
        *slot = vertices.as_ptr();
        let result = render(atomic);
        *slot = original;
        result
    }
}

unsafe extern "C" fn destroy_skin(skin: *mut ()) -> *mut () {
    SKIN_VERTICES.lock().unwrap().remove(&(skin as usize));
    let destroy: unsafe extern "C" fn(*mut ()) -> *mut () =
        unsafe { std::mem::transmute(0x00575070usize) };
    unsafe { destroy(skin) }
}

unsafe extern "C" fn clone_skin(dst: *mut (), src: *mut (), offset: i32, size: i32) -> *mut () {
    unsafe {
        let clone: unsafe extern "C" fn(*mut (), *mut (), i32, i32) -> *mut () =
            std::mem::transmute(0x00575600usize);
        let result = clone(dst, src, offset, size);
        let mut cache = SKIN_VERTICES.lock().unwrap();
        if let Some(vertices) = cache.get(&(get_skin(src) as usize)).cloned() {
            let skin = get_skin(dst);
            if !skin.is_null() {
                cache.insert(skin as usize, vertices);
                *dst.byte_add(0x48).cast::<usize>() = render_atomic as *const () as usize;
            }
        }
        result
    }
}

// RenderWare matrices have a metadata word after each three-component vector.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Matrix {
    right: Vec3,
    flags: u32,
    up: Vec3,
    pad1: u32,
    at: Vec3,
    pad2: u32,
    pos: Vec3,
    pad3: u32,
}

impl Matrix {
    fn transform(&self, point: Vec3) -> Vec3 {
        std::array::from_fn(|axis| {
            point[0] * self.right[axis]
                + point[1] * self.up[axis]
                + point[2] * self.at[axis]
                + self.pos[axis]
        })
    }
}

#[derive(Clone, Copy)]
struct BoneScale {
    to_bone: Matrix,
    from_bone: Matrix,
    scale: Vec3,
}

impl BoneScale {
    fn transform(&self, point: Vec3) -> Vec3 {
        let local = self.to_bone.transform(point);
        self.from_bone
            .transform(std::array::from_fn(|axis| local[axis] * self.scale[axis]))
    }
}

fn deform_vertex(
    original: Vec3,
    indices: [u8; 4],
    weights: [f32; 4],
    bones: &[Option<BoneScale>],
) -> Vec3 {
    let mut result = [0.0; 3];
    for (index, weight) in indices.into_iter().zip(weights) {
        // Unused influence slots can contain indices outside the bone palette.
        if weight == 0.0 {
            continue;
        }
        let point = match bones.get(index as usize).and_then(Option::as_ref) {
            Some(bone) => bone.transform(original),
            None => original,
        };
        for axis in 0..3 {
            result[axis] += point[axis] * weight;
        }
    }
    result
}

fn deform_cache(
    original: &[Vec3],
    indices: &[[u8; 4]],
    weights: &[[f32; 4]],
    bones: &[Option<BoneScale>],
) -> Vec<Vec3> {
    // Normals, when present, follow the positions in this allocation. Keep
    // them unchanged, matching the Xbox position-only deformation pass.
    let mut result = original.to_vec();
    for (index, (&indices, &weights)) in indices.iter().zip(weights).enumerate() {
        result[index] = deform_vertex(original[index], indices, weights, bones);
    }
    result
}

// The game's skin plugin stores an 80-byte record per bone. Its first 64 bytes
// are the inverse bind matrix; hierarchy flags are at +0x48.
#[repr(C)]
struct SkinBone {
    inverse_bind: Matrix,
    metadata: [u32; 2],
    hierarchy_flags: u32,
    metadata2: u32,
}

fn parent_indices(bones: &[SkinBone]) -> Option<Vec<usize>> {
    let mut parents = Vec::with_capacity(bones.len());
    let mut stack = Vec::new();
    let mut parent = 0; // Xbox uses bone zero when the parent index is negative.
    for (index, bone) in bones.iter().enumerate() {
        parents.push(parent);
        // The final node can pop the implicit root sentinel, but has no
        // successor whose parent needs to be resolved.
        if index + 1 == bones.len() {
            break;
        }
        if bone.hierarchy_flags & 2 != 0 {
            stack.push(index);
        }
        parent = if bone.hierarchy_flags & 1 != 0 {
            parents[stack.pop()?]
        } else {
            index
        };
    }
    Some(parents)
}

unsafe extern "C" fn scale_atomic(atomic: *mut (), scaling: *const ()) -> *mut () {
    unsafe {
        if atomic.is_null() || scaling.is_null() {
            return atomic;
        }
        let geometry = *atomic.byte_add(0x18).cast::<*mut ()>();
        if geometry.is_null() {
            return atomic;
        }

        let skin = get_skin(atomic);
        if skin.is_null() {
            return atomic;
        }
        let bone_count = *skin.cast::<i32>();
        let vertex_count = *geometry.byte_add(0x20).cast::<i32>();
        if !(1..=64).contains(&bone_count) || vertex_count <= 0 {
            return atomic;
        }
        let skin_bones = *skin.byte_add(4).cast::<*const SkinBone>();
        let indices = *skin.byte_add(0x1c).cast::<*const [u8; 4]>();
        let weights = *skin.byte_add(0x20).cast::<*const [f32; 4]>();
        let morph_target = *geometry.byte_add(0x4c).cast::<*mut ()>();
        if skin_bones.is_null() || indices.is_null() || weights.is_null() || morph_target.is_null()
        {
            return atomic;
        }
        let vertices = *skin.byte_add(0x24).cast::<*const Vec3>();
        let render = *atomic.byte_add(0x48).cast::<usize>();
        if vertices.is_null()
            || *skin.byte_add(0xc).cast::<i32>() != vertex_count
            || (render != 0x00577590 && render != render_atomic as *const () as usize)
        {
            return atomic;
        }

        let get_geometry_data: unsafe extern "C" fn(*mut ()) -> *const () =
            std::mem::transmute(0x004fea00usize);
        let geometry_data = get_geometry_data(geometry);
        if geometry_data.is_null() {
            return atomic;
        }
        let id_count = *geometry_data.byte_add(0x28).cast::<u32>() as usize;
        let ids = *geometry_data.byte_add(0x2c).cast::<*const u32>();
        if ids.is_null() || id_count != bone_count as usize {
            return atomic;
        }

        let skin_bones = std::slice::from_raw_parts(skin_bones, bone_count as usize);
        let Some(parents) = parent_indices(skin_bones) else {
            return atomic;
        };
        let ids = std::slice::from_raw_parts(ids, id_count);
        let get_scale: unsafe extern "thiscall" fn(*const (), u32, *mut Vec3) -> i32 =
            std::mem::transmute(0x005037d0usize);
        let invert_matrix: unsafe extern "C" fn(*mut Matrix, *const Matrix) -> *mut Matrix =
            std::mem::transmute(0x0055d7e0usize);

        let mut bones = Vec::with_capacity(id_count);
        for (index, id) in ids.iter().enumerate() {
            let mut scale = [1.0; 3];
            if get_scale(scaling, *id, &mut scale) == 0 {
                bones.push(None);
                continue;
            }
            // Scale around the parent joint, including Xbox's root fallback.
            let to_bone = skin_bones[parents[index]].inverse_bind;
            let mut from_bone = Matrix::default();
            invert_matrix(&mut from_bone, &to_bone);
            bones.push(Some(BoneScale {
                to_bone,
                from_bone,
                scale,
            }));
        }

        let has_normals = !(*morph_target.byte_add(0x18).cast::<*const Vec3>()).is_null();
        let buffer_len = vertex_count as usize * if has_normals { 2 } else { 1 };
        let deformed = deform_cache(
            std::slice::from_raw_parts(vertices, buffer_len),
            std::slice::from_raw_parts(indices, vertex_count as usize),
            std::slice::from_raw_parts(weights, vertex_count as usize),
            &bones,
        );
        SKIN_VERTICES
            .lock()
            .unwrap()
            .insert(skin as usize, deformed.into());
        *atomic.byte_add(0x48).cast::<usize>() = render_atomic as *const () as usize;
        atomic
    }
}

pub fn init() {
    unsafe {
        // All three patches are required: appearance scaling plus the clone
        // and destruction paths that manage our separate PC skinning cache.
        let site = 0x00503fbb as *mut ();
        let expected = [0x68, 0xd0, 0x27, 0x50, 0x00]; // push 0x5027d0
        if std::slice::from_raw_parts(site.cast::<u8>(), expected.len()) != expected
            || std::slice::from_raw_parts(0x0057520d as *const u8, 5)
                != [0x68, 0x00, 0x56, 0x57, 0x00]
            || std::slice::from_raw_parts(0x005755de as *const u8, 5)
                != [0xe8, 0x8d, 0xfa, 0xff, 0xff]
        {
            logger::log(
                LogLevel::Warn,
                "Skater weight fix: unsupported scaling/skin callbacks; skipped",
            );
            return;
        }
        patch::patch_u32(site.byte_add(1), scale_atomic as *const () as u32);
        patch::patch_u32(0x0057520e as *mut (), clone_skin as *const () as u32);
        patch::patch_call(0x005755de as *mut (), destroy_skin as *const ());
    }
}

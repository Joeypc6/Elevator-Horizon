// The elevators with a camouflage in each section, from the game's thread.
use super::*;
use std::collections::HashMap;
use std::sync::RwLock;

static SECTIONS: RwLock<Option<HashMap<(i32, i32, i32), Vec<([i32; 3], u32)>>>> = RwLock::new(None);

impl Mesher<'_> {
    /// Every camouflage now: (position, state).
    pub fn hz_elevator_set_camos(camos: &[([i32; 3], u32)]) {
        let Ok(mut table) = SECTIONS.write() else { return };
        let mut by_section: HashMap<(i32, i32, i32), Vec<([i32; 3], u32)>> = HashMap::new();
        for (p, state) in camos {
            by_section.entry((p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16))).or_default().push((*p, *state));
        }
        *table = Some(by_section);
    }
}

pub(super) fn swap(pad: &mut Padded, region: &[[Option<&Chunk>; 3]; 3], sy: i32) {
    let Ok(table) = SECTIONS.read() else { return };
    let Some(table) = table.as_ref() else { return };
    if table.is_empty() {
        return;
    }
    let Some(center) = region[1][1] else { return };
    let base_y = center.min_y() + sy * 16;
    let section_y = base_y.div_euclid(16);
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let Some(list) = table.get(&(center.x + dx, section_y + dy, center.z + dz)) else { continue };
                for (p, state) in list {
                    let (x, y, z) = (p[0] - center.x * 16, p[1] - base_y, p[2] - center.z * 16);
                    if (-B..16 + B).contains(&x) && (-B..16 + B).contains(&y) && (-B..16 + B).contains(&z) {
                        pad.blocks[pi(x, y, z)] = *state;
                    }
                }
            }
        }
    }
}

use bevy::{
    prelude::*,
};

// Our own plugin:
pub struct JayObserve;

impl Plugin for JayObserve {
    fn build(&self, app: &mut App) {
        app
            .add_system(observation_system_update_cells)
            .add_system(observation_system_update_observed.after(observation_system_update_cells))
            .add_system(observation_system_update_hashmap.after(observation_system_update_observed));
    }
}

#[derive(Component, Debug, Default)]
pub struct Observable {
    pub cell: usize,
    pub observed: Vec<Entity>,
}

// A resource which collects observable thingies by spatial hashing.
pub struct StuffsToObserve {
    stuff: Vec<Vec<Entity>>,
    cell_size: f32,
    width: usize,
    depth: usize,
}

impl StuffsToObserve {
    pub fn new(width: usize, depth: usize, cell_size: f32) -> StuffsToObserve {
        let mut stuff = Vec::new();
        let size = width * depth;
        for _ in 0..size {
            stuff.push(Vec::new());
        }
        StuffsToObserve {
            stuff,
            cell_size,
            width,
            depth,
        }
    }
}

impl StuffsToObserve {
    fn collect_cells(&self, cell: usize) -> Vec<usize>
    {
        let mut all_cells = Vec::new();

        all_cells.push(cell);

        let w = self.width as isize;

        // the eight surrounding cells
        let cell_r = cell as isize + 1;
        let cell_l = cell as isize - 1;
        let cell_u = cell as isize + w;
        let cell_d = cell as isize - w;
        let cell_ur = cell as isize + w + 1;
        let cell_ul = cell as isize + w - 1;
        let cell_dr = cell as isize - w + 1;
        let cell_dl = cell as isize - w - 1;

        if ind_valid(&self.stuff, cell_l)
        { all_cells.push(cell_l as usize); }

        if ind_valid(&self.stuff, cell_r)
        { all_cells.push(cell_r as usize); }

        if ind_valid(&self.stuff, cell_u)
        { all_cells.push(cell_u as usize); }

        if ind_valid(&self.stuff, cell_d)
        { all_cells.push(cell_d as usize); }

        if ind_valid(&self.stuff, cell_ur)
        { all_cells.push(cell_ur as usize); }

        if ind_valid(&self.stuff, cell_ul)
        { all_cells.push(cell_ul as usize); }

        if ind_valid(&self.stuff, cell_dr)
        { all_cells.push(cell_dr as usize); }

        if ind_valid(&self.stuff, cell_dl)
        { all_cells.push(cell_dl as usize); }

        all_cells
    }
}

fn ind_valid<T>(set: &Vec<T>, ind: isize) -> bool
{
    if ind < 0 { return false; }
    if ind >= set.len() as isize { return false; }

    true
}

// Our crude spatial-hash function.
fn hash_function(pos: Vec3, cell_size: f32, width: usize, depth: usize) -> usize
{
    if cell_size <= 0.
    { return 0; }

    let x = (f32::floor(pos.x / cell_size) as usize).clamp(0, width - 1);
    let y = (f32::floor(pos.y / cell_size) as usize).clamp(0, depth - 1);
    x + y * width
    // (f32::floor(pos.x / cell_size) + f32::floor(pos.y / cell_size) * width as f32) as usize
}

fn observation_system_update_cells(
    stuff_to_observe: Res<StuffsToObserve>,
    mut observables: Query<(&mut Observable, &Transform)>)
{
    for (mut obs, transform) in observables.iter_mut() {
        obs.cell = hash_function(transform.translation, stuff_to_observe.cell_size, stuff_to_observe.width, stuff_to_observe.depth);
    }
}

fn observation_system_update_observed(
    stuff_to_observe: Res<StuffsToObserve>,
    mut observables: Query<&mut Observable>)
{
    for mut obs in observables.iter_mut() {
        obs.observed.clear();
        let near_cells = stuff_to_observe.collect_cells(obs.cell);
        for near_cell in near_cells.iter()
        {
            for entity in stuff_to_observe.stuff[*near_cell].iter() {
                obs.observed.push(*entity);
            }
        }
    }
}

fn observation_system_update_hashmap(
    mut stuff_to_observe: ResMut<StuffsToObserve>,
    observables: Query<(&Observable, Entity)>)
{
    for thing in stuff_to_observe.stuff.iter_mut() {
        thing.clear();
    }
    for (obs, entity) in observables.iter() {
        if let Some(set) = stuff_to_observe.stuff.get_mut(obs.cell)
        {
            set.push(entity);
        }
    }
}
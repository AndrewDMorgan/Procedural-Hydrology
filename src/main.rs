use std::mem;
use std::time::Instant;
use sdl2::render::{TextureAccess, TextureCreator};
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::WindowContext;
use sdl2::rect::Rect;

mod events;


pub fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    
    // Creating a window
    let (window_start_width, window_start_height) = (1512, 1080);  // the river/height map size
    let (window_start_width, window_start_height) = (750, 750);  // the river size
    let mut window = video
        .window("Name of Game (todo!)", window_start_width, window_start_height)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    window.set_minimum_size(100, 100)
        .map_err(|e| e.to_string())?;
    
    // --- Create an SDL2 surface and texture ---
    let (_device_width, _device_height) = (video.desktop_display_mode(0)?.w, video.desktop_display_mode(0)?.h);
    let mut window_surface = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    
    // creating the texture that all runtime drawing will be done to
    // this texture will then be uploaded onto the window_surface
    let texture_creator: TextureCreator<WindowContext> = window_surface.texture_creator();
    let mut surface_texture = texture_creator
        .create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, window_start_width, window_start_height)
        .map_err(|e| e.to_string())?;
    
    let mut event_pump = sdl.event_pump()?;
    let mut events = events::Events::new();
    
    
    let very_beginning_start = Instant::now();
    
    
    let mut chunks = vec![vec![0; window_start_height as usize]; window_start_width as usize];
    
    
    
    // generating continental masses
    //    - using a cellular automata:  (kinda gem of life but wayyyyy worse)
    //  1. generate tons of points
    //  2. points around many points survive
    //  2. points with few points have a high chance of death
    // this should generate small and larger islands
    //  3. any islands greater than a set size become "continents";    this doesn't matter as the rivers have to be a certain distance from the shore anyway
    
    let start = Instant::now();
    for _ in 0..(window_start_width * window_start_height / 25) as usize {
        let (x, y) = (
            rand::random_range(0..window_start_width ) as usize,
            rand::random_range(0..window_start_height) as usize,
        );
        chunks[x][y] = 255;  // for now being binary for this first pass
    }
    
    // doing .. iterations of the automata for now
    for iteration in 0..6 {
        let mut chunks_new = vec![vec![0; window_start_height as usize]; window_start_width as usize];
        for x in 0..window_start_width {
            for y in 0..window_start_height {
                let mut count = 0;
                let r = (12 - (iteration as isize - 2)*(iteration as isize - 3)) as u32;  // the radius for gather the count of solid tiles
                let starting_search = (x.saturating_sub(r) as usize, y.saturating_sub(r) as usize);
                let ending_search = ((x + r + 1).min(window_start_width) as usize, (y + r + 1).min(window_start_height) as usize);
                for xp in starting_search.0..ending_search.0 {
                    for yp in starting_search.1..ending_search.1 {
                        if chunks[xp][yp] > 0 { count += 1; }
                    }
                }
                
                let square_size = (ending_search.0 - starting_search.0 + 1) * (ending_search.1 - starting_search.1 + 1);
                if count < rand::random_range(square_size / 20..square_size / 9 + 1) { chunks_new[x as usize][y as usize] = 0; }
                else { chunks_new[x as usize][y as usize] = 255; }
            }
        }
        chunks = chunks_new;
    }
    
    // cleaning up straggling bits
    for x in 0..window_start_width as usize {
        for y in 0..window_start_height as usize {
            let mut count = 0;
            if x > 0 { count += chunks[x-1][y].min(1); }
            if y > 0 { count += chunks[x][y-1].min(1); }
            if x < window_start_width as usize - 1 { count += chunks[x+1][y].min(1); }
            if y < window_start_height as usize - 1 { count += chunks[x][y+1].min(1); }
            if count < 2 { chunks[x][y] = 0; }
        }
    }
    println!("Land Generation Took {:?}", start.elapsed());
    
    
    
    // generating a distance field from the shoreline (I'm sure this won't murder my computer..... although it can be rough and approximate)
    //  - for speed imma do a simple light flooding like algorithm slowly spreading the light from the edges outwards
    //       * this isn't 100% accurate, but is close enough probably
    
    // all land tiles next to ocean become a 1, every other remains the same, then the expansions begin
    let start = Instant::now();
    // using a hashmap to prevent overlapping or directly next to each other rivers, to make it slightly less chaotic
    let mut starting_points = std::collections::HashMap::new();
    let mut future_tiles_current = std::collections::HashMap::new();
    for x in 0..window_start_width as usize {
        for y in 0..window_start_height as usize {
            if chunks[x][y] == 0 { continue; }  // a water tile (distance is 0)
            let mut count = 0;
            if x > 0 { count += 1-chunks[x-1][y].min(1); }
            if y > 0 { count += 1-chunks[x][y-1].min(1); }
            if x < window_start_width as usize - 1 { count += 1-chunks[x+1][y].min(1); }
            if y < window_start_height as usize - 1 { count += 1-chunks[x][y+1].min(1); }
            if count == 0 { continue; }  // not next to water
            future_tiles_current.insert((x, y), 1);
        }
    }
    let mut future_tiles_future = std::collections::HashMap::new();
    while !future_tiles_current.is_empty() {
        future_tiles_future.clear();
        for ((x, y), distance) in &future_tiles_current {
            chunks[*x][*y] = *distance;
            if *x > 0 && *y > 0 && (*x as u32) < window_start_width-2 && (*y as u32) < window_start_height-2 && *distance > 35 && rand::random_range(0.0..1.0) < 0.02 {
                // the divisor represents the chunk size (too lazy to manage the magic numbers rn)
                starting_points.insert((*x/10, *y/10), (*x, *y));
            }
            if *x > 0 && chunks[*x-1][*y] == 255 {
                future_tiles_future.insert((*x-1, *y), (*distance + 1).min(*future_tiles_future.get(&(*x-1, *y)).unwrap_or(&255u8)));
            }
            if *y > 0 && chunks[*x][*y-1] == 255 {
                future_tiles_future.insert((*x, *y-1), (*distance + 1).min(*future_tiles_future.get(&(*x, *y-1)).unwrap_or(&255u8)));
            }
            if *x < window_start_width as usize - 1 && chunks[*x+1][*y] == 255 {
                future_tiles_future.insert((*x+1, *y), (*distance + 1).min(*future_tiles_future.get(&(*x+1, *y)).unwrap_or(&255u8)));
            }
            if *y < window_start_height as usize - 1 && chunks[*x][*y+1] == 255 {
                future_tiles_future.insert((*x, *y+1), (*distance + 1).min(*future_tiles_future.get(&(*x, *y+1)).unwrap_or(&255u8)));
            }
        }
        // double buffering to reduce memory allocations (maybe 10%ish faster? but took 5mins, so whatever)
        mem::swap(&mut future_tiles_current, &mut future_tiles_future);
    }
    let starting_points = starting_points.values().collect::<Vec<_>>();
    println!("Distance Mask Generation (And River Locations) Took {:?}\n  {} Starts", start.elapsed(), starting_points.len());
    
    for points in &starting_points {
        //chunks[points.0][points.1] = 255;  // just for visualization (maybe center them on more mountainous bands that were identified?)
    }
    
    // going through each river point and descending the gradient towards the ocean (not height map, but distance field)
    //      - if a river hits another they conjoin making the following segments larger than current
    //      - random "lakes" can be put along these rivers, which are just super thick nodes within that single spot in the river
    //      - if a river node is increased to a value beyond a certain amount, and is within near enough range of the ocean:
    // * randomly create a split in the paths with equal strength or biased to create river deltas
    
    // oh no..... this is the fun part.........
    // this data setup will most certainly change, but for now should be good enough ig
    // size (thickness), inertia (f32; 0-1), distance from source, x velocity (f32; 0-1) y velocity (f32; 0-1), unique id
    let start = Instant::now();
    let mut rivers: Vec<Vec<Option<(u32, f32, u32, f32, f32, u32)>>> = vec![vec![None; window_start_height as usize]; window_start_width as usize];
    let mut river_paths: Vec<Vec<(usize, usize)>> = vec![];  // all positions in a river, tied to it's id
    let mut i = 0;
    // vel_x_f, vel_y_f, id, merged id's (to not merge twice)
    let mut river_state: Vec<Option<(f32, f32, u32, Vec<u32>)>> = vec![];
    for &(mut x, mut y) in starting_points {  // todo! make rivers join together, add lakes, and ig that sort of stuff
        // Initialize velocity based on gradient
        let grad_x = -((chunks[x + 1][y] as f32 - chunks[x - 1][y] as f32) / 2.0);
        let grad_y = -((chunks[x][y + 1] as f32 - chunks[x][y - 1] as f32) / 2.0);
        
        let mut vx = grad_x;
        let mut vy = grad_y;
        let length = (vx * vx + vy * vy).sqrt();
        if length > 0.001 {
            vx /= length;
            vy /= length;
        }
        rivers[x][y] = Some((
            1,
            0.075,
            0,
            vx,
            vy,
            i,
        ));
        river_paths.push(vec![(x, y)]);
        river_state.push(Some((x as f32 + 0.5, y as f32 + 0.5, i, vec![i])));
        i += 1;
    }
    
    // pre allocating to hopefully save some time repeatedly reallocating the same exact thing
    let mut rivers_near: Vec<(f32, f32, usize, f32)> = Vec::with_capacity(64);
    
    loop {
        let dead = river_state.iter_mut().map(|d| {
            match d {
                Some((x_f, y_f, id, merged_ids)) => {
                    let (x, y) = (*x_f as usize, *y_f as usize);
                    
                    // this movement math is gonna be awful, look away physics nerds
                    match rivers.get(x).and_then(|row| row.get(y)) {
                        Some(Some((size, inertia, dist, x_vel, y_vel, id))) => {
                            if *dist > 750 { return true; }  // too long, probably got stuck
                            
                            // find the gradient.......... WHYYYYYYYYYY, NO MORE MATH PLZ    (this is now a tomorrow issue.... could not be bothered with this rn; correction, ai provided the grad math! yay!)
                            let mut true_grad_x = -((chunks[x + 1][y] as f32 - chunks[x - 1][y] as f32) / 2.0);  // negated sign to ensure flow downhill
                            let mut true_grad_y = -((chunks[x][y + 1] as f32 - chunks[x][y - 1] as f32) / 2.0);
                            let grad_len = (true_grad_x * true_grad_x + true_grad_y * true_grad_y).sqrt();
                            if grad_len > 0.001 {
                                true_grad_x /= grad_len;
                                true_grad_y /= grad_len;
                            }
                            let mut grad_x = true_grad_x;
                            let mut grad_y = true_grad_y;
                            // adding random jitter
                            grad_x += rand::random_range(-1.0..1.0) * (1.0 / *size as f32) * 0.5;
                            grad_y += rand::random_range(-1.0..1.0) * (1.0 / *size as f32) * 0.5;
                            let grad_len = (grad_x * grad_x + grad_y * grad_y).sqrt();
                            if grad_len > 0.001 {
                                grad_x /= grad_len;
                                grad_y /= grad_len;
                            }
                            
                            // biasing the new vel towards any nearby rivers, based on their size
                            let mut total_sizes = 0;  // used to normalize it
                            // dir x, dir y, size, dst
                            rivers_near.clear();
                            let r = 8;  // the radius of the search
                            let start_position = (x.saturating_sub(r+1)+1, y.saturating_sub(r+1)+1);  // the - r - 1 + 1 is to keep it going from 1 -> width - 1 so it isn't ever on the very final cell of an edge
                            let end_position = ((x + r).min(window_start_width as usize - 2), (y + r).min(window_start_height as usize - 2));
                            for xp in start_position.0..end_position.0 {
                                for yp in start_position.1..end_position.1 {
                                    match rivers[xp][yp] {
                                        Some((size, _inertia, _dist, _x_vel, _y_vel, _id)) => {
                                            // also the length for the direction vector
                                            let dx = xp as f32 - x as f32;
                                            let dy = yp as f32 - y as f32;
                                            
                                            // checking the dot product to make sure it's not against the grad
                                            if true_grad_x * dx + true_grad_y * dy < 0.0 { continue; }
                                            
                                            let dst = (dx * dx + dy * dy).sqrt();
                                            let dx = dx / dst;  // normalized
                                            let dy = dy / dst;
                                            if dst < 1.0 { continue; }  // on top of the river
                                            rivers_near.push((dx, dy, size as usize, dst));
                                            total_sizes += size as usize;
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            
                            let x_vel_comp = (*x_vel * 0.2 + grad_x) * 0.5 + (
                                rivers_near.iter().map(|river| river.0 * (river.2 as f32 / total_sizes as f32 / river.3)).sum::<f32>()
                            ) * 0.5;
                            let y_vel_comp = (*y_vel * 0.2 + grad_y) * 0.5 + (
                                rivers_near.iter().map(|river| river.1 * (river.2 as f32 / total_sizes as f32 / river.3)).sum::<f32>()
                            ) * 0.5;
                            
                            
                            // as inertia -> 1 x_vel_new -> x_vel
                            // as inertia -> 0 x_vel_new -> grad_x
                            // more magic numbers I can't be bother to create parameters for!!!
                            let new_vx = (x_vel_comp * 0.8) * (1.0 - *inertia) + *x_vel * *inertia;
                            let new_vy = (y_vel_comp * 0.8) * (1.0 - *inertia) + *y_vel * *inertia;
                            
                            
                            let length = (new_vx * new_vx + new_vy * new_vy).sqrt();
                            if length < 0.001 { return true; }  // Stagnant river     shouldn't happen?
                            let new_x = match *x_f + new_vx / length * (1.0 + (0.5 - (*x_f - (*x_f).floor())).abs()) {
                                x if x >= 1.0 => { x },  // the 1.0 is to keep it from crashing at the bounds
                                _ => { return true; }
                            };
                            let new_y = match *y_f + new_vy / length * (1.0 + (0.5 - (*y_f - (*y_f).floor())).abs()) {
                                y if y >= 1.0 => { y },
                                _ => { return true; }
                            };
                            *x_f = new_x;
                            *y_f = new_y;
                            let new_x = *x_f as usize;
                            let new_y = *y_f as usize;
                            river_paths[*id as usize].push((new_x, new_y));  // adding now so all rivers connect even if they converge, so at least that convergence is known
                            if new_x >= window_start_width as usize - 1 || new_y >= window_start_height as usize - 1 { return true; }
                            if new_x == x && new_y == y { return true; }  // got stuck??
                            if chunks[new_x][new_y] == 0 { return true; }  // hit the ocean!!!
                            let mut size = *size;
                            let inertia = *inertia;
                            let dist = *dist;
                            let id = *id;
                            if rivers[new_x][new_y].is_some() {
                                let (river_dst, river_size, river_id) = match &rivers[new_x][new_y] {
                                    Some(river) => { (river.2, river.0, river.5) },
                                    _ => { (0, 0, 0) },
                                };
                                if !merged_ids.contains(&river_id) {
                                    merged_ids.push(river_id);
                                    for river_position in &river_paths[river_id as usize][river_dst as usize + 1..] {
                                        if river_position.0 >= window_start_width as usize || river_position.1 >= window_start_height as usize { continue; }
                                        let current_size = match &rivers[river_position.0][river_position.1] {
                                            Some((s, ..)) => { *s },
                                            _ => { continue; }
                                        };
                                        
                                        let river = rivers[river_position.0][river_position.1].unwrap();
                                        rivers[river_position.0][river_position.1] = Some((
                                            current_size + size,
                                            river.1 * 0.9 + 0.05,
                                            river.2,
                                            river.3 * 0.8 + new_vx * 0.2,  // merging behavior is going on in here
                                            river.4 * 0.8 + new_vy * 0.2,
                                            river.5
                                        ));
                                    }
                                    size += river_size;
                                }
                            }
                            rivers[new_x][new_y] = Some((
                                size,
                                inertia,
                                dist + 1,
                                new_vx,
                                new_vy,
                                id  // for tracking river convergence
                            ));
                        },
                        _ => {
                            // shouldn't run I think? so mostly just for safety?
                            return true;
                        },
                    }
                    
                    false
                },
                _ => { true }
            }
        }).collect::<Vec<bool>>();
        river_state.iter_mut()
            .enumerate()
            .for_each(|(i, state)| {
                match dead[i] { false => {}, true => *state = None, };
            });
        if dead.iter().all(|x| *x) { break; }
    }
    println!("River Path Generation Took {:?}", start.elapsed());
    
    // not sure if this is possible since it'd reroute rivers too much      OR I don't join rivers until after since they already bias towards each other?   and keep copies!! now they can be bent correctly!
    // use a rough cornering approximation to bend curves more in flatter regions and straighten steep slopes
    
    // should I just do d/dx^2 for each river, or should I combine local rivers first to ensure they still somewhat stay connected?
    // Small islands in large rivers wouldn't be super unrealistic though, so idk it may just work out
    
    let start = Instant::now();
    for path in &mut river_paths {
        let mut gradients = vec![];
        for point_index in 1..path.len() - 1 {
            let grad_x = path[point_index].0 as f32 - path[point_index - 1].0 as f32;
            let grad_y = path[point_index].1 as f32 - path[point_index - 1].1 as f32;
            let mut len = (grad_x*grad_x + grad_y*grad_y).sqrt();
            if len < 0.01 { len = 1.0 }  // to prevent crazyyyyy scaling
            assert!((grad_x / len).abs() <= 1.0 && (grad_y / len).abs() <= 1.0);  // since it's magnitude is 1.0, all components are <= 1.0 if correctly set
            gradients.push((grad_x / len, grad_y / len));
        }
        if gradients.len() < 2 { continue; }
        let mut acceleration_grads = vec![(0.0, 0.0), (0.0, 0.0)];  // padded such that the first two points at least have something and aren't miss aligned, preventing messy indexes
        for point_index in 1..gradients.len() - 1 {
            let grad_x = gradients[point_index].0 - gradients[point_index - 1].0;
            let grad_y = gradients[point_index].1 - gradients[point_index - 1].1;
            let mut len = (grad_x*grad_x + grad_y*grad_y).sqrt();
            if len < 0.01 { len = 1.0 }  // to prevent crazyyyyy scaling
            assert!((grad_x / len).abs() <= 1.0 && (grad_y / len).abs() <= 1.0);  // since it's magnitude is 1.0, all components are <= 1.0 if correctly set
            acceleration_grads.push((grad_x / len, grad_y / len));
        }
        // adding padding to the end to prevent index errors at the end
        acceleration_grads.push((0.0, 0.0));
        acceleration_grads.push((0.0, 0.0));
        gradients.insert(0, (0.0, 0.0));  // re-aligning the velocities
        gradients.push((0.0, 0.0));  // re-aligning the velocities while not messing up the acceleration calculations
        for (index, point) in path.iter_mut().enumerate() {
            // as this var goes to 0.0, the acceleration becomes more perpendicular to the motion vector (i.e. a turn, rather than acceleration straight down)
            let perpendicularity =
                gradients[index].0 * acceleration_grads[index].0 +
                gradients[index].1 * acceleration_grads[index].1;
            assert!(perpendicularity.abs() <= 1.11);  // ummmm....... something is clearly borked lol
            assert!((acceleration_grads[index].0 * perpendicularity.abs()).abs() <= 1.0);
            assert!((acceleration_grads[index].1 * perpendicularity.abs()).abs() <= 1.0);
            // once again, the 0.5 is to prevent bias to the left
            point.0 = (point.0 as f32 + 0.5 + acceleration_grads[index].0 * (1.0 - perpendicularity.abs()) * 0.75) as usize;
            point.1 = (point.1 as f32 + 0.5 + acceleration_grads[index].1 * (1.0 - perpendicularity.abs()) * 0.75) as usize;
        }
    }
    println!("Generated Gradients And Bended Rivers In {:?}", start.elapsed());
    
    
    // using the river, generate a distance field from them
    
    let start = Instant::now();
    let mut river_dst = vec![vec![9999; window_start_height as usize]; window_start_width as usize];
    for x in 0..window_start_width {
        for y in 0..window_start_height {
            let point = (x as usize, y as usize);
            match rivers[point.0][point.1] {
                Some(..) if chunks[point.0][point.1] > 0 => {
                    river_dst[point.0][point.1] = 1usize;
                },
                _ if chunks[point.0][point.1] == 0 => {
                    river_dst[point.0][point.1] = 9999;  // should make it non land and non river
                },
                _ if chunks[point.0][point.1] > 0 => {
                    river_dst[point.0][point.1] = 0;
                },
                _ => {
                    river_dst[point.0][point.1] = 9999;  // debugging, shouldn't happen?
                }
            }
        }
    }
    
    // using a hashmap to prevent overlapping or directly next to each other rivers, to make it slightly less chaotic
    let mut future_tiles_current = std::collections::HashMap::new();
    for x in 0..window_start_width as usize {
        for y in 0..window_start_height as usize {
            if river_dst[x][y] != 1 || chunks[x][y] == 0 { continue; }  // a river or ocean tile (distance is 0)
            future_tiles_current.insert((x, y), 1);
        }
    }
    let mut future_tiles_future = std::collections::HashMap::new();
    while !future_tiles_current.is_empty() {
        future_tiles_future.clear();
        for ((x, y), distance) in &future_tiles_current {
            river_dst[*x][*y] = *distance;
            if *x > 0 && river_dst[*x-1][*y] == 0 {
                future_tiles_future.insert((*x-1, *y), (*distance + 1).min(*future_tiles_future.get(&(*x-1, *y)).unwrap_or(&255usize)));
            }
            if *y > 0 && river_dst[*x][*y-1] == 0 {
                future_tiles_future.insert((*x, *y-1), (*distance + 1).min(*future_tiles_future.get(&(*x, *y-1)).unwrap_or(&255usize)));
            }
            if *x < window_start_width as usize - 1 && river_dst[*x+1][*y] == 0 {
                future_tiles_future.insert((*x+1, *y), (*distance + 1).min(*future_tiles_future.get(&(*x+1, *y)).unwrap_or(&255usize)));
            }
            if *y < window_start_height as usize - 1 && river_dst[*x][*y+1] == 0 {
                future_tiles_future.insert((*x, *y+1), (*distance + 1).min(*future_tiles_future.get(&(*x, *y+1)).unwrap_or(&255usize)));
            }
        }
        // double buffering to reduce memory allocations (maybe 10%ish faster? but took 5mins, so whatever)
        mem::swap(&mut future_tiles_current, &mut future_tiles_future);
    }
    println!("Generate Distances To Rivers In {:?}", start.elapsed());
    
    
    // combine the river mask and the land mask
    //      - the land mask should purely slope down terrain near the immediate shore, the rivers
    //           should have a larger trend to allow more accurate sloping
    
    // I'm guessing a river height map is now needed? to combine the maps
    
    // generate a constantly increasing, but randomly sloped (within reason based on river size and distance from the coast)..
    // ..river height for each river node
    // *   ∫x=0,x->nodes.len() (node.height) == final height    wonderful integrals/summations...... have fun future me
    // but..... nearby rivers have to remain similar.... so more of a 2d height map is necessary, but only one that increases? this gonna be fun
    // should a flood fill be used for this too lol? just flood fill and add perlin noise ranging from 0->n where n is a positive float?
    // that should create a centered mass around the center of the continents that's the highest.
    // a map would be needed to allow slower growth near the ocean (not always) to allow flood plains and deltas
    
    
    
    
    
    // generate the land height
    
    
    
    
    
    
    // merge the land based on the distance mask
    //    - it should purely move terrain up to meet it
    //    - if the terrain is already higher, based on a mask it can create canyons or slope slowly downwards
    //          **** for a simple model at first maybe lerp or add the height of the river based on the mask? ******
    //   * note that canyons should only happen in areas calculated to be relatively flat/consistently sloped
    //   * canyons should also only happen in largish-massive rivers and not near the sea
    
    
    println!("Total Elapsed Time: {:?}", very_beginning_start.elapsed());
    
    
    'running: loop {
        events.update();
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'running,
                sdl2::event::Event::KeyDown { keycode, .. } => {
                    if let Some(key) = keycode {
                        events.update_down(key)
                    }
                },
                sdl2::event::Event::KeyUp { keycode, .. } => {
                    if let Some(key) = keycode {
                        events.update_up(key)
                    }
                },
                _ => {}
            }
        }
        
        // !====! Do Rendering Here! !====!
        
        // rendering
        // creating a pixel buffer to pass around to reduce draw calls as the cpu is faster than repeatedly waiting for the gpu to return data
        // the gpu is fast, but data moves between the gpu and cpu slowly
        let _buffer_result = surface_texture.with_lock(None, |pixels, pitch| {
            // rendering in here
            
            
            // rendering the height map for now
            for (x, row) in river_dst.iter().enumerate() {
                for (y, height) in row.iter().enumerate() {
                    pixels[x * 3 + (y) * pitch + 0] = (*height).min(255) as u8;
                    pixels[x * 3 + (y) * pitch + 1] = (*height).min(255) as u8;
                    pixels[x * 3 + (y) * pitch + 2] = (*height).min(255) as u8;
                }
            }
            
            
        })?;
        
        // !====! No Rendering Beyond Here !====!
        
        // clearing and drawing the texture
        window_surface.clear();
        let window_size = window_surface.window().size();
        window_surface.copy(&surface_texture, None, Rect::new(0, 0, window_size.0, window_size.1))?;
        
        // flushing the screen and stuff
        window_surface.present();
    }
    
    Ok(())
}

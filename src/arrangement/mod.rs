use crate::graph::Graph;

pub struct Coordinates {
    pub x: f64,
    pub y: f64,
}
pub struct Point {
    pub id: u64,
    pub coordinates: Coordinates,
}

pub struct Arrangement {
    // pub points: Vec<Point>,
    pub points: Vec<f32>,
    pub lines: Vec<(Point, Point)>,
}

fn to_coordinates(bin_index: usize, node_index: usize) -> Coordinates {
    // TODO(Menno 14.12.2024) the X coordinate should probably be slightly offset for each node_index
    //  Maybe following a curve, so that lines connecting nodes in the same bin do not overlap.
    let x = bin_index as f64 * 0.015 - 0.99;
    let y = node_index as f64 * 0.003 - 0.99;

    Coordinates { x, y }
}
impl Arrangement {
    pub fn new(graph: &Graph) -> Arrangement {
        let mut arrangement: Arrangement = Arrangement {
            points: Vec::new(),
            lines: Vec::new(),
        };

        #[derive(Clone, Copy)]
        struct BinEntry {
            pub distance_from_start: u32,
            pub id: u64,
        }

        let mut bins: Vec<Vec<BinEntry>> =
            vec![Vec::new(); graph.max_distance_to_solution as usize + 1];

        // We group each node based on their distance from the solution.
        for (key, node) in graph.map.iter() {
            bins[node.distance_to_solution.unwrap() as usize].push(BinEntry {
                distance_from_start: node.distance_to_start.unwrap(),
                id: *key,
            });
        }

        for (bin_index, bin) in bins.iter_mut().enumerate() {
            println!("bin_index: {} has {} points", bin_index, bin.len());

            // Within each group, the nodes are sorted by their distance from the start.
            bin.sort_by(|a, b| a.distance_from_start.cmp(&b.distance_from_start));
            for (node_index, bin_entry) in bin.iter().enumerate() {
                // Point {
                //   id: bin_entry.id,
                //   coordinates: ),
                // }
                let coords = to_coordinates(bin_index, node_index);
                arrangement.points.push(coords.x as f32);
                arrangement.points.push(coords.y as f32);

            }
        }
        arrangement
    }
}

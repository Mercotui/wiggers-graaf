// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board::BoardId;
use crate::graph::Graph;
use std::cmp::max;

pub struct Arrangement {
    pub width: u32,
    pub height: u32,
    pub points: Vec<f32>,
}

impl Arrangement {
    pub fn new(graph: &Graph, active_state: BoardId) -> Arrangement {
        let mut arrangement: Arrangement = Arrangement {
            width: graph.max_distance_to_solution + 1,
            height: 0,
            points: Vec::new(),
        };

        #[derive(Clone, Copy)]
        struct BinEntry {
            pub distance_from_start: u32,
            pub id: BoardId,
        }

        let mut bins: Vec<Vec<BinEntry>> = vec![Vec::new(); arrangement.width as usize];

        // We group each node based on their distance from the solution.
        for (key, node) in graph.map.iter() {
            bins[node.distance_to_solution.unwrap() as usize].push(BinEntry {
                distance_from_start: node.distance_to_start.unwrap(),
                id: *key,
            });
        }

        for (bin_index, bin) in bins.iter_mut().enumerate() {
            arrangement.height = max(arrangement.height, bin.len() as u32);
            println!("bin_index: {} has {} points", bin_index, bin.len());

            // Within each group, the nodes are sorted by their distance from the start.
            bin.sort_by(|a, b| a.distance_from_start.cmp(&b.distance_from_start));
            for (node_index, bin_entry) in bin.iter().enumerate() {
                // Add point's coordinates
                arrangement.points.push(bin_index as f32);
                arrangement.points.push(node_index as f32);

                // Add point's size
                arrangement.points.push(if bin_entry.id == active_state {
                    6.0
                } else {
                    3.0
                });

                // Add point's color
                if bin_entry.id == active_state {
                    arrangement.points.push(1.0);
                    arrangement.points.push(0.27);
                    arrangement.points.push(0.23);
                } else {
                    arrangement.points.push(0.0);
                    arrangement.points.push(0.0);
                    arrangement.points.push(0.0);
                }
            }
        }
        arrangement
    }
}

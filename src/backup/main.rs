
use std::collections::HashMap;
use std::cmp::Ordering;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
    label: String,
}

fn euclidean_distance(p1: &Point, p2: &Point) -> f64 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}

fn k_nearest_neighbors(k: usize, point: &Point, dataset: &[Point]) -> String {
    let mut distances: Vec<(f64, &Point)> = dataset
        .iter()
        .map(|p| (euclidean_distance(&point, p), p))
        .collect();

    distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    let mut label_count = HashMap::new();

    for &(_, neighbor) in distances.iter().take(k) {
        *label_count.entry(&neighbor.label).or_insert(0) += 1;
    }

    let mut sorted_count = Vec::from_iter(label_count);
    sorted_count.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

    sorted_count[0].0.clone().to_string()
}

fn main() {
    let dataset = vec![
        Point { x: 1.0, y: 1.0, label: "A".to_string() },
        Point { x: 2.0, y: 2.0, label: "A".to_string() },
        Point { x: 1.0, y: 2.0, label: "A".to_string() },
        Point { x: 5.0, y: 5.0, label: "B".to_string() },
        Point { x: 6.0, y: 6.0, label: "B".to_string() },
        Point { x: 5.0, y: 6.0, label: "B".to_string() },
        Point { x: -1.0, y: 1.0, label: "C".to_string() },
    ];

    let test_point = Point { x: 3.0, y: 3.0, label: "".to_string() };
    let k = 3;
    let label = k_nearest_neighbors(k, &test_point, &dataset);

    println!("The label of the test point is: {}", label);
}

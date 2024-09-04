use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use position_share::{rdp, Coordinate, Positions, Search};
use uuid::Uuid;

/// Generates a simulated path for an object.
///
/// This function creates a 3D path that represents the movement of an object
/// over time. The path is generated using parametric equations to simulate
/// realistic movement.
///
/// # Arguments
///
/// * `num_points` - The number of points to generate along the path.
///
/// # Returns
///
/// A `Positions` object containing the generated path points with their
/// corresponding timestamps.
#[allow(clippy::suboptimal_flops)]
fn generate_path(num_points: usize) -> Positions {
    let mut positions = Positions::default();
    let start_time = Utc::now();

    for i in 0..num_points {
        // Calculate the parametric variable t, which ranges from 0 to 1
        #[allow(clippy::cast_precision_loss)]
        let t = i as f64 / num_points as f64;

        // Simulate a 3D curve representing more complex movement
        // X-coordinate: Combination of sinusoidal motions with different frequencies
        // and amplitudes
        let x = 1000.0 * t.sin() + 300.0 * (3.0 * t).cos() + 150.0 * (5.0 * t).sin();
        // Y-coordinate: Combination of sinusoidal and polynomial functions
        let y = 500.0 * (2.0 * t).sin() + 200.0 * t.powi(2) - 100.0 * (4.0 * t).cos();
        // Z-coordinate (depth): Combination of quadratic function and sinusoidal motion
        // This creates a more complex depth variation
        let z = -100.0 * t.powi(2) + 50.0 * (3.0 * t).sin();

        // Create a new coordinate point
        let coordinate = Coordinate::new(x, y, z);
        // Calculate the timestamp for this point (one point every 10 seconds)
        #[allow(clippy::cast_possible_wrap)]
        let timestamp = start_time + Duration::seconds(i as i64 * 10);

        // Add the coordinate and timestamp to the Positions object
        positions.add(timestamp, coordinate);
    }

    positions
}

fn bench_most_novel_coordinates(c: &mut Criterion) {
    let positions = generate_path(5000);
    let recipient = Uuid::new_v4();
    c.bench_function("most_novel_coordinates", |b| {
        b.iter(|| {
            positions.most_novel_coordinates(
                &Search::new(rdp, Some(0.4)),
                black_box(&recipient),
                black_box(100),
            )
        });
    });
}

criterion_group!(benches, bench_most_novel_coordinates);
criterion_main!(benches);

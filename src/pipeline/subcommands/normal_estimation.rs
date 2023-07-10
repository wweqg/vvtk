use clap::Parser;
use std::ops::Sub;
use crate::pipeline::channel::Channel;
use crate::pipeline::PipelineMessage;
use crate::formats::{PointCloud, pointxyzrgba::PointXyzRgba, pointxyzrgbanormal::PointXyzRgbaNormal};

use super::Subcommand;

#[derive(Parser)]
#[clap(
    about = "Performs normal estimation on point clouds.",
)]
pub struct Args {
    #[clap(short, long, default_value = "1.0")]
    radius: f64,
}

pub struct NormalEstimation {
    args: Args,
}

impl NormalEstimation {
    pub fn from_args(args: Vec<String>) -> Box<dyn Subcommand> {
        Box::from(NormalEstimation {
            args: Args::parse_from(args),
        })
    }
}

impl Subcommand for NormalEstimation {
    fn handle(&mut self, messages: Vec<PipelineMessage>, channel: &Channel) {
        // Perform normal estimation for each point cloud in the messages
        for message in messages {
            match message {
                PipelineMessage::IndexedPointCloud(pc, i) => {
                    let normal_estimation_result = perform_normal_estimation(&pc, self.args.radius);
                    channel.send(PipelineMessage::IndexedPointCloudNormal(normal_estimation_result, i));
                }
                PipelineMessage::Metrics(_) | PipelineMessage::IndexedPointCloudNormal(_, _) | PipelineMessage::DummyForIncrement => {}
                PipelineMessage::End => {
                    channel.send(message);
                }
            }
        }
    }
}

fn perform_normal_estimation(pc: &PointCloud<PointXyzRgba>, radius: f64) -> PointCloud<PointXyzRgbaNormal> {
    // Select Neighboring Points
    let neighbors = select_neighboring_points(pc, radius);

    // // Compute Covariance Matrix
    // let covariance_matrices = compute_covariance_matrices(&cleaned_cloud, &neighbors);

    // // Compute Eigenvalues and Eigenvectors
    // let eigen_results = compute_eigenvalues_and_eigenvectors(&covariance_matrices);

    // // Assign Normal Vector
    // let normals = assign_normal_vectors(&eigen_results);

    // // Complete Normal Estimation
    // let normal_estimation_result = complete_normal_estimation(&cleaned_cloud, &neighbors, &normals);

    // normal_estimation_result
    let point = PointXyzRgbaNormal {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        r: 255,
        g: 0,
        b: 0,
        a: 255,
        normal_x: 0.0,
        normal_y: 0.0,
        normal_z: 1.0,
    };
    let point_cloud = PointCloud {
        number_of_points: 1,
        points: vec![point],
    };
    point_cloud
}

fn select_neighboring_points(pc: &PointCloud<PointXyzRgba>, radius: f64) -> Vec<Vec<usize>> {
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); pc.number_of_points];

    for i in 0..pc.number_of_points {
        let mut point_neighbors: Vec<usize> = Vec::new();
        let p1 = &pc.points[i];

        for j in 0..pc.number_of_points {
            if i != j {
                let p2 = &pc.points[j];
                let dist = distance(&[p1.x, p1.y, p1.z], &[p2.x, p2.y, p2.z]);

                if dist <= radius {
                    point_neighbors.push(j);
                }
            }
        }

        neighbors[i] = point_neighbors;
    }

    neighbors
}


fn distance<T>(p1: &[T; 3], p2: &[T; 3]) -> f64
where
    T: Sub<Output = T> + Into<f64> + Copy,
{
    let dx = (p1[0] - p2[0]).into();
    let dy = (p1[1] - p2[1]).into();
    let dz = (p1[2] - p2[2]).into();

    (dx * dx + dy * dy + dz * dz).sqrt()
}

// fn compute_covariance_matrices(pc: &PointCloud<PointXyzRgba>, neighbors: &[Vec<usize>]) -> Vec<CovarianceMatrix> {
//     // Compute the covariance matrix for each point and its neighbors
//     // Return a vector containing the covariance matrices
// }

// fn compute_eigenvalues_and_eigenvectors(covariance_matrices: &[CovarianceMatrix]) -> Vec<EigenResult> {
//     // Compute the eigenvalues and eigenvectors for each covariance matrix
//     // Return a vector containing the eigenvalue and eigenvector results
// }

// fn assign_normal_vectors(eigen_results: &[EigenResult]) -> Vec<NormalVector> {
//     // Assign the normal vector for each point based on the eigenvector corresponding to the smallest eigenvalue
//     // The normal vector can be derived from the eigenvector
//     // Return a vector containing the assigned normal vectors
// }

// fn complete_normal_estimation(
//     pc: &PointCloud<PointXyzRgba>,
//     neighbors: &[Vec<usize>],
//     normals: &[NormalVector],
// ) -> PointCloud<NormalVector> {
//     // After traversing all points in the point cloud and propagating the orientations,
//     // you will have estimated a normal vector for each point with orientations consistent across the entire point cloud
//     // Return the completed normal estimation as a new point cloud
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_select_neighboring_points() {
        // Create a sample point cloud
        let points = vec![
            PointXyzRgba { x: 0.0, y: 0.0, z: 0.0, r: 0, g: 0, b: 0, a: 255 },
            PointXyzRgba { x: 1.0, y: 1.0, z: 1.0, r: 255, g: 255, b: 255, a: 255 },
            PointXyzRgba { x: 2.0, y: 2.0, z: 2.0, r: 255, g: 0, b: 0, a: 255 },
            PointXyzRgba { x: 3.0, y: 3.0, z: 3.0, r: 0, g: 255, b: 0, a: 255 },
            PointXyzRgba { x: 4.0, y: 4.0, z: 4.0, r: 0, g: 0, b: 255, a: 255 },
        ];
    
        let pc = PointCloud {
            number_of_points: points.len(),
            points,
        };
    
        let radius = 3.0; // Example radius value
    
        let neighbors = select_neighboring_points(&pc, radius);
    
        // Assert the expected neighbors for each point
    
        // Point 0 should have neighbors 1
        assert_eq!(neighbors[0], vec![1]);
    
        // Point 1 should have neighbors 0, 2
        assert_eq!(neighbors[1], vec![0, 2]);
    
        // Point 2 should have neighbors 1, 3
        assert_eq!(neighbors[2], vec![1, 3]);
    
        // Point 3 should have neighbors 2, 4
        assert_eq!(neighbors[3], vec![2, 4]);
    
        // Point 4 should have neighbors 3
        assert_eq!(neighbors[4], vec![3]);
    }    
}


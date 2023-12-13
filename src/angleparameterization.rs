// Add your relevant imports here
use super::evaluate::Evaluate;
use super::parameterization::Parameterization;

#[derive(Debug, Clone)]
pub struct AngleParameterization {
    pub total_angles: Vec<f64>,
}

impl AngleParameterization {
    pub fn from<T: Evaluate>(evaluable: &T, iterations: i32) -> Self {
        let mut output = Vec::new();
        let mut prev_tangent = evaluable.tangent_at(0.0);
        let mut total_angle = 0.0;

        output.push(total_angle);

        let mut i = 1;
        while i < iterations + 1 {
            let t = i as f64 / iterations as f64;
            let current_tangent = evaluable.tangent_at(t);

            // Compute the angle between the current and previous tangents
            let angle = f64::abs(prev_tangent.angle(current_tangent));

            total_angle += angle;
            output.push(total_angle);

            prev_tangent = current_tangent;
            i = i + 1;
        }

        Self {
            total_angles: output,
        }
    }

    pub fn get_total_angle(&self) -> f64 {
        *self.total_angles.last().unwrap()
    }

    fn search_for_index(&self, target: f64) -> usize {
        let mut left = 0;
        let mut right = self.total_angles.len() - 1;

        while left < right {
            let middle = (right + left) / 2;

            if left == middle {
                return middle;
            }
            if right == middle {
                return left;
            }
            if self.total_angles[middle] == target {
                return middle;
            }

            if self.total_angles[middle] < target {
                left = middle;
            } else {
                right = middle;
            }
        }

        panic!("Couldn't find the target angle!")
    }

    pub fn get_angle_from_t(&self, t: f64) -> f64 {
        let fractional_index = t * (self.total_angles.len() - 1) as f64;
        let index = fractional_index as usize;
        let fraction = fractional_index - index as f64;

        let angle_start = self.total_angles[index];
        let angle_end = if index != self.total_angles.len() - 1 {
            self.total_angles[index + 1]
        } else {
            1.
        };
        let segment_angle = angle_end - angle_start;

        angle_start + segment_angle * fraction
    }

    pub fn find_parameters_for_angle_intervals(&self, angle_interval: f64) -> Vec<f64> {
        let mut curve_parameters = Vec::new();
        let mut next_angle_threshold = angle_interval;

        for (i, &total_angle) in self.total_angles.iter().enumerate() {
            if total_angle >= next_angle_threshold {
                let prev_angle = self.total_angles[i - 1];
                let angle_diff = total_angle - prev_angle;

                let fraction = (next_angle_threshold - prev_angle) / angle_diff;
                let curve_parameter =
                    (i as f64 - 1.0 + fraction) / (self.total_angles.len() as f64 - 1.0);

                curve_parameters.push(curve_parameter);

                // Set the next angle threshold
                next_angle_threshold += angle_interval;
            }
        }
        curve_parameters
    }

    pub fn find_parameters_for_angle_intervals_in_range(
        &self,
        angle_interval: f64,
        min_t: f64,
        max_t: f64,
    ) -> Vec<f64> {
        println!("min_t: {}, max_t: {}", min_t, max_t);
        assert!(min_t >= 0.0 && min_t <= 1.0);
        assert!(max_t >= 0.0 && max_t <= 1.0);
        let mut curve_parameters = Vec::new();

        // Convert min_t and max_t to corresponding indices in total_angles array
        let min_index = (min_t * (self.total_angles.len() - 1) as f64).round() as usize;
        let max_index = (max_t * (self.total_angles.len() - 1) as f64).round() as usize;

        // Initialize next_angle_threshold based on the first angle in the selected range
        let mut next_angle_threshold = self.total_angles[min_index] + angle_interval;

        // Loop through the restricted range of total_angles
        for i in (min_index + 1)..=max_index {
            let total_angle = self.total_angles[i];

            if total_angle >= next_angle_threshold {
                let prev_angle = self.total_angles[i - 1];
                let angle_diff = total_angle - prev_angle;

                // Interpolate to find the t value at which the angle crosses the threshold
                let fraction = (next_angle_threshold - prev_angle) / angle_diff;
                let curve_parameter =
                    (i as f64 - 1.0 + fraction) / (self.total_angles.len() as f64 - 1.0);

                curve_parameters.push(curve_parameter);

                // Update the next angle threshold
                next_angle_threshold += angle_interval;
            }
        }
        curve_parameters
    }
}

impl Parameterization for AngleParameterization {
    fn parameterize(&self, u: f64) -> f64 {
        let target_angle = u * self.get_total_angle();
        let angle_index = self.search_for_index(target_angle);

        if target_angle == self.total_angles[angle_index] {
            return angle_index as f64 / (self.total_angles.len() - 1) as f64;
        } else {
            let angle_start = self.total_angles[angle_index];
            let angle_end = self.total_angles[angle_index + 1];
            let segment_angle = angle_end - angle_start;

            let segment_fraction = (target_angle - angle_start) / segment_angle;

            return (angle_index as f64 + segment_fraction) / (self.total_angles.len() - 1) as f64;
        }
    }
}

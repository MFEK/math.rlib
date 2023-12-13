pub trait Subdivide {
    fn split(&self, t: f64) -> Option<(Self, Self)>
    where
        Self: Sized;

    fn split_at_multiple_t(&self, mut t_values: Vec<f64>) -> Vec<Self>
    where
        Self: Sized + Clone,
    {
        // Sort the t-values to make it easier to split the curve
        t_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut beziers: Vec<Self> = Vec::new();
        let mut last_t: f64 = 0.0;

        // Store the current segment; initially, it's the whole Bezier curve
        let mut current_bezier = self.clone();

        for &t in &t_values {
            // Normalize t to the remaining part of the curve
            let local_t = (t - last_t) / (1.0 - last_t);

            // Perform the split
            if let Some((left, right)) = current_bezier.split(local_t) {
                // Store the left (first) part of the split
                beziers.push(left);

                // Update the current segment to be the right (second) part of the split
                current_bezier = right;

                // Update the last t-value
                last_t = t;
            }
        }

        // Add the remaining part of the curve
        beziers.push(current_bezier);

        beziers
    }
}

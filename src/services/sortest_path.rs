use log::{error, info, trace};
use ocl::{Buffer, Context, Device, Kernel, MemFlags, Platform, Program, Queue, Result, SpatialDims};

use crate::models::Matrix;

/// The kernel to use
const OPENCL_PROGRAM: &'static str = r#"
__kernel void initialize_algorithm_buffers(__global float *result, __global float *distance, __global int *visited) {
    // Get the global id based on count of vertexs and assigned for thread
    int gid = get_global_id(0);

    // Initialize the buffers in parallel
    if (gid == 0) {
        distance[gid] = 0;
        visited[gid] = 1;
        result[gid] = 0;
    } else {
        distance[gid] = 0;
        visited[gid] = 0;
        result[gid] = FLT_MAX;
    }
}

__kernel void shortest_path_algorithm(__global float *result, __global float *matrix, __global float *distance, __global int *visited, int vertex_count) {
    // Get the global id based on count of vertexs and assigned for thread
    int gid = get_global_id(0);

    // Validate if the vertex is not visited
    if (visited[gid] != 1) {
        // Mark the vertex as visited
        visited[gid] = 1;

        // Get the start edge
        for(int edge = 0; edge < vertex_count; edge++) {
            // Get the edge from adjacent matrix
            float weight = matrix[gid * vertex_count + edge];

            // Validate if the edge is valid
            if (weight != 0.0f && weight != FLT_MAX) {
                // Get the distance
                float dist = result[edge] + weight;

                // Get the result
                if (distance[gid] == 0.0 || result[gid] > dist) {
                    distance[gid] = dist;
                }
            }
        }
    }
}

__kernel void merge_sortest_path(__global float *result, __global float *distance, __global int *visited) {
    // Get the global id based on count of vertexs and assigned for thread
    int gid = get_global_id(0);

    // Get the result
    if (result[gid] > distance[gid]) {
        result[gid] = distance[gid];
    }

    // Reset the visited flag
    if (gid != 0) {
        visited[gid] = 0;
    }
}
"#;

/// The sortest path service
///
/// # Fields
///
/// * `device` - The device to use
/// * `context` - The context to use
/// * `program` - The program to use
/// * `queue` - The queue to use
///
pub struct SortestPath {
    queue: Queue,
    device: Device,
    context: Context,
    program: Program,
}

impl SortestPath {
    /// Create a new instance of the walker
    ///
    /// # Returns
    ///
    /// * `Walker` - The walker object
    ///
    pub fn new() -> SortestPath {
        // Get the default platform
        let platform = Platform::default();

        // Print the initialization info
        trace!("Initializing OpenCL components before operations...");
        info!("Using platform: {}", platform.name().unwrap());

        // Prepare OpenCL Elements
        let device = Device::first(platform).unwrap();
        let context = Context::builder().devices(device).build().unwrap();
        let queue = Queue::new(&context, device, None).unwrap();
        let program = Program::builder()
            .src(OPENCL_PROGRAM).devices(device)
            .build(&context).unwrap();

        // Print the device info and start operations
        trace!("Initialized OpenCL components, starting operations...");
        info!("Using device: {}", device.name().unwrap());

        // Build the object for the service
        SortestPath { device, context, program, queue }
    }
}

impl SortestPath {
    /// Process the kernel result
    ///
    /// This method will process the result of the kernel and return the result
    /// of the operation.
    ///
    /// # Arguments
    ///
    /// * `result` - The result of the kernel
    /// * `lambda` - The lambda to execute if the result is ok
    ///
    /// # Returns
    ///
    /// * `Result<Vec<f32>>` - The result of the operation
    ///
    fn process_kernel_result(&self, result: Result<()>, lambda: impl FnOnce() -> Result<Vec<f32>>) -> Result<Vec<f32>> {
        match result {
            Ok(_) => lambda(),
            Err(e) => {
                // Print the error in console
                error!("Critical error occurred in kernel: {}", e.api_status().unwrap().to_string());

                // Return error code
                return Err(e);
            }
        }
    }

    /// Returns the best path for hamiltonian walk
    ///
    /// This method will return the best path for the hamiltonian walk
    /// enumerating elements in step order over the vector result.
    ///
    /// # Arguments
    ///
    /// * `matrix` - The matrix to walk
    ///
    /// # Returns
    ///
    /// * `Vec<i32>` - The path of the walk
    ///
    pub  fn get_sortest_path(&self, matrix: Matrix) -> Result<Vec<f32>> {
        // Print the initialization
        trace!("Initializing buffers for the kernel...");

        // Instantiate the result vector
        let mut result = vec![0.0; matrix.width];
        let mut distance = vec![0.0; matrix.width];

        unsafe {
            // Instantiate the matrix as buffer
            let matrix_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.data.len())
                .flags(MemFlags::READ_ONLY).use_host_slice(&matrix.data)
                .build().unwrap();

            // Instantiate result matrix
            let result_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Instantiate path matrix
            let distance_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Instantiate visited matrix
            let visited_buffer = Buffer::<i32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Print the buffers initialization
            trace!("Buffers initialized, starting kernel...");

            // Instantiate the buffers kernel
            let initialize_algorithm_buffers = Kernel::builder()
                .program(&self.program).queue(self.queue.clone())
                .name("initialize_algorithm_buffers").global_work_size(SpatialDims::One((matrix.width) as usize))
                .arg(&result_buffer).arg(&distance_buffer).arg(&visited_buffer)
                .build().unwrap();

            // Instantiate the main kernel
            let shortest_path_algorithm = Kernel::builder()
                .program(&self.program).queue(self.queue.clone())
                .name("shortest_path_algorithm").global_work_size(SpatialDims::One((matrix.width) as usize))
                .arg(&result_buffer).arg(&matrix_buffer).arg(&distance_buffer).arg(&visited_buffer).arg(matrix.width as i32)
                .build().unwrap();

            // Instantiate the merge kernel
            let merge_sortest_path = Kernel::builder()
                .program(&self.program).queue(self.queue.clone())
                .name("merge_sortest_path").global_work_size(SpatialDims::One((matrix.width) as usize))
                .arg(&result_buffer).arg(&distance_buffer).arg(&visited_buffer)
                .build().unwrap();

            // Print the kernel start
            trace!("Kernel started, enqueueing the operation...");

            // Run the program and wait for it to finish
            self.process_kernel_result(initialize_algorithm_buffers.enq(), || {
                // Run the algorithm
                for _ in 0..matrix.width {
                    self.process_kernel_result(shortest_path_algorithm.enq(), || {
                        self.process_kernel_result(merge_sortest_path.enq(), || {
                            // Print the end of the operation
                            trace!("The iteration was successfully completed! Preparing the return of result...");

                            // Copy the results to the host
                            result_buffer.read(&mut result).enq()?;
                            distance_buffer.read(&mut distance).enq()?;

                            // Print the result
                            trace!("Result: {:?}", result);
                            trace!("Distance: {:?}", distance);

                            // Return dummy result
                            Ok(vec![0.0; 1])
                        })
                    }).expect("Error while merging the sortest path");
                }

                // Return the result
                Ok(result)
            })
        }
    }
}

impl Drop for SortestPath {
    fn drop(&mut self) {
        drop(&self.device);
        drop(&self.context);
        drop(&self.program);
        drop(&self.queue);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sortest_path() {
        // Prepare the matrix
        let matrix = Matrix::new(6, 6,vec![
            01.0, 04.0, 02.0, 00.0, 00.0, 00.0,
            04.0, 01.0, 01.0, 05.0, 00.0, 00.0,
            02.0, 01.0, 01.0, 08.0, 10.0, 00.0,
            00.0, 05.0, 08.0, 01.0, 02.0, 06.0,
            00.0, 00.0, 10.0, 02.0, 01.0, 02.0,
            00.0, 00.0, 00.0, 06.0, 02.0, 01.0
        ]);

        // Prepare the expected result
        let expected = vec![0.0, 3.0, 2.0, 8.0, 10.0, 12.0];

        // Get the result
        let result = SortestPath::new().get_sortest_path(matrix);

        // Check if the result is correct
        assert_eq!(result.unwrap(), expected);
    }
}

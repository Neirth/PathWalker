use log::{error, info, trace};
use ocl::{Buffer, Context, Device, Kernel, MemFlags, Platform, Program, Queue, Result, SpatialDims};

use crate::models::{Matrix, PathResult};

/// The kernel to use
const OPENCL_PROGRAM: &'static str = r#"
__kernel void initialize_algorithm_buffers(__global float *result, __global float *distance, __global int *visited, __global float *vertex, __global float *vertex_temp) {
    // Get the global id based on count of vertexs and assigned for thread
    int gid = get_global_id(0);

    // Initialize the buffers in parallel
    if (gid == 0) {
        visited[gid] = 1;
        result[gid] = 0;
    } else {
        visited[gid] = 0;
        result[gid] = FLT_MAX;
    }

    distance[gid] = 0;
    vertex[gid] = 0;
    vertex_temp[gid] = 0;
}

__kernel void shortest_path_algorithm(__global float *result, __global float *matrix, __global float *distance, __global int *visited, __global float *vertex_temp, int vertex_count) {
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
                    vertex_temp[gid] = edge;
                }
            }
        }
    }
}

__kernel void merge_sortest_path(__global float *result, __global float *distance, __global int *visited, __global float *vertex, __global float *vertex_temp) {
    // Get the global id based on count of vertexs and assigned for thread
    int gid = get_global_id(0);

    // Get the result
    if (result[gid] > distance[gid]) {
        result[gid] = distance[gid];
        vertex[gid] = vertex_temp[gid];
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
    fn process_kernel_result(&self, result: Result<()>, lambda: impl FnOnce() -> Result<Vec<PathResult>>) -> Result<Vec<PathResult>> {
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
    pub  fn get_sortest_path(&self, matrix: Matrix) -> Result<Vec<PathResult>> {
        // Print the initialization
        trace!("Initializing buffers for the kernel...");

        // Instantiate the result vector
        let mut result = vec![0.0; matrix.width];
        let mut vertex = vec![0.0; matrix.width];
        let mut distance = vec![0.0; matrix.width];

        unsafe {
            // Instantiate the matrix as buffer
            let matrix_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.data.len())
                .flags(MemFlags::READ_ONLY).use_host_slice(&matrix.data)
                .build().unwrap();

            // Instantiate result vector as buffer
            let result_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Instantiate distance vector as buffer
            let distance_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Instantiate vertex vector as buffer
            let vertex_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Instantiate vertex temp vector as buffer
            let vertex_temp_buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Instantiate visited vector as buffer
            let visited_buffer = Buffer::<i32>::builder()
                .queue(self.queue.clone()).len(matrix.width)
                .build().unwrap();

            // Print the buffers initialization
            trace!("Buffers initialized, starting kernel...");

            // Instantiate the buffers kernel
            let initialize_algorithm_buffers = Kernel::builder()
                .program(&self.program).queue(self.queue.clone())
                .name("initialize_algorithm_buffers").global_work_size(SpatialDims::One((matrix.width) as usize))
                .arg(&result_buffer).arg(&distance_buffer).arg(&visited_buffer).arg(&vertex_buffer).arg(&vertex_temp_buffer)
                .build().unwrap();

            // Instantiate the main kernel
            let shortest_path_algorithm = Kernel::builder()
                .program(&self.program).queue(self.queue.clone())
                .name("shortest_path_algorithm").global_work_size(SpatialDims::One((matrix.width) as usize))
                .arg(&result_buffer).arg(&matrix_buffer).arg(&distance_buffer).arg(&visited_buffer).arg(&vertex_temp_buffer)
                .arg(matrix.width as i32).build().unwrap();

            // Instantiate the merge kernel
            let merge_sortest_path = Kernel::builder()
                .program(&self.program).queue(self.queue.clone())
                .name("merge_sortest_path").global_work_size(SpatialDims::One((matrix.width) as usize))
                .arg(&result_buffer).arg(&distance_buffer).arg(&visited_buffer).arg(&vertex_buffer).arg(&vertex_temp_buffer)
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
                            vertex_buffer.read(&mut vertex).enq()?;
                            distance_buffer.read(&mut distance).enq()?;

                            // Print the result
                            trace!("Result: {:?}", result);
                            trace!("Distance: {:?}", distance);
                            trace!("Vertex: {:?}", vertex);

                            // Return dummy result
                            Ok(vec![PathResult(0, 0.0)])
                        })
                    }).expect("Error while merging the sortest path");
                }

                // Compute the result path vector
                let mut result_path = Vec::<PathResult>::new();
                for x in 0..matrix.width {
                    result_path.push(PathResult(vertex[x] as i32, distance[x]));
                }

                // Print the result of the operation
                info!("Path to walk {:?}", result_path);

                // Return the result
                Ok(result_path)
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
        let expected = vec![
            PathResult(0, 0.0),
            PathResult(2, 3.0),
            PathResult(0, 2.0),
            PathResult(1, 8.0),
            PathResult(3, 10.0),
            PathResult(4, 12.0)
        ];

        // Get the result
        let result = SortestPath::new().get_sortest_path(matrix);

        // Check if the result is correct
        assert_eq!(result.unwrap(), expected);
    }
}

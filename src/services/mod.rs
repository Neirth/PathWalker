use log::{error, info, trace};
use crate::models::Matrix;
use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue, Result, SpatialDims,};

/// The kernel to use
const KERNEL: &'static str = r#"
__kernel void get_hamiltonian_path(__global float *result, __global float *matrix, __global float *path, int size) {

}
"#;

/// The walker service
///
/// # Fields
///
/// * `device` - The device to use
/// * `context` - The context to use
/// * `program` - The program to use
/// * `queue` - The queue to use
pub struct Walker {
    queue: Queue,
    device: Device,
    context: Context,
    program: Program,
}

impl Walker {
    /// Create a new instance of the walker
    ///
    /// # Returns
    ///
    /// * `Walker` - The walker object
    pub fn new() -> Walker {
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
            .src(KERNEL).devices(device)
            .build(&context).unwrap();

        // Print the device info and start operations
        trace!("Initialized OpenCL components, starting operations...");
        info!("Using device: {}", device.name().unwrap());

        // Build the object for the service
        Walker { device, context, program, queue }
    }
}

impl Walker {
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
    pub fn get_walk_path(&self, matrix: Matrix) -> Result<Vec<f32>> {
        // Print the initialization
        trace!("Initializing buffers for the kernel...");

        // Instantiate result matrix
        let result_buffer = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(matrix.width).build().unwrap();

        // Instantiate the matrix as buffer
        let matrix_buffer = Buffer::<f32>::builder()
            .len(matrix.data.len()).queue(self.queue.clone())
            .copy_host_slice(&matrix.data)
            .build().unwrap();

        // Instantiate path matrix
        let path_buffer = Buffer::<f32>::builder()
            .queue(self.queue.clone())
            .len(matrix.width).build().unwrap();

        // Print the buffers initialization
        trace!("Buffers initialized, starting kernel...");

        // Instantiate a queue for the kernel to use
        let kernel = Kernel::builder()
            .program(&self.program).queue(self.queue.clone())
            .name("get_hamiltonian_path").global_work_size(SpatialDims::One((matrix.width) as usize))
            //.local_work_size(SpatialDims::One(matrix.width as usize))
            .arg(&result_buffer).arg(&matrix_buffer).arg(&path_buffer)
            .arg((matrix.width * matrix.height) as i32).build().unwrap();

        // Print the kernel start
        trace!("Kernel started, enqueueing the operation...");

        // Run the program and wait for it to finish
        unsafe {
            match kernel.enq() {
                Ok(_) => trace!("The operation was successfully completed! Returning to controller..."),
                Err(e) => {
                    // Print the error in console
                    error!("Critical error occurred in kernel: {}", e.api_status().unwrap().to_string());

                    // Return error code
                    return Err(e);
                }
            }
        }

        // Copy the result to the host
        let mut result = vec![0.0; matrix.width];
        result_buffer.read(&mut result).enq()?;

        // Return the result
        Ok(result)
    }
}

impl Drop for Walker {
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
    fn test_get_walk_path() {
        // Prepare the matrix
        let matrix = Matrix::new(3, 3, vec![
            0.0, 1.0, 0.0,
            1.0, 0.0, 1.0,
            0.0, 1.0, 0.0
        ]);

        // Prepare the expected result
        let expected = vec![1.0, 2.0, 3.0];

        // Get the result
        let result = Walker::new().get_walk_path(matrix);

        // Check if the result is correct
        assert_eq!(result.unwrap(), expected);
    }
}

<p align="center">
  <img src="https://github.com/Neirth/PathWalker/blob/master/assets/logo.svg?raw=true">
</p>

PathWalker is a Rust project that aims to explore the effective parallelism of heavy operations and how to leverage them in a demanding environment like a data center, through the use of OpenCL.

## Problem

The main issue that PathWalker addresses is how to speed up certain processes that might otherwise be quite costly on the wrong device. In most servers, all computing power is derived to the processor, which is shared by different cores that must access shared time with other cores of other operating systems. The services that make this interaction possible can take up CPU time, and queries to external computer services can cause additional delays, which in turn wastes the capacity of the machine.

The GPGPU concept has been introduced to relieve devices of the responsibility of having to manage an operating system, or in the worst case, a Type 1 Hypervisor running different operating systems, to launch workflows waiting for an answer quickly.

## OpenCL and CPU Coordination

OpenCL is a heterogeneous computing framework that works with a queuing system, allowing the CPU to delegate the workload to the intensive processing unit effectively. This frees up the CPU to take care of other tasks without having to worry about processing requests full-time. OpenCL also allows memory regions that the processing unit will have available to be managed from the application, either by transferring complete memory arrays to it or exploiting those of the host itself.

OpenCL's programming language is based on the C language, but it has no access to libraries and external libraries such as Boost. This is done to make code compatible with the largest amount of hardware that has certified drivers.

## Building and Running

To build PathWalker, beforehand, we must installed the OpenCL Headers and their driver for you accelerator device, after this, use the following command:

```bash
cargo build --release
```

To run PathWalker, use the following command:

```bash
cargo run --release
```

## Deploying with Docker

PathWalker has a Dockerfile available to deploy it, and we recommend this option as the primary one. To deploy it, use the following command:

```bash
docker build -t pathwalker .
```

This dockerfile also install a Portable Computing Language (PoCL) driver for OpenCL, so you can run PathWalker on any device with a CPU.

## API Documentation

The endpoints available are:

* `POST /shortest`: Returns the shortest path between two points in a graph. Example

    Request Example:
    ```json
    {
      "data": [8, 2, 3, 4, 5, 6, 7, 1, 9, 10, 11, 12, 13, 14, 15, 16],
      "width": 4,
      "height": 4
    }
    ```
    
    Response Example:
    ```json
    {
      "path": [[8, 1], [7, 1], [6, 1], [5, 1]],
      "status": "ok"
    }
    ```


## License

PathWalker is licensed under the MIT License. See [LICENSE](LICENSE) for more information.
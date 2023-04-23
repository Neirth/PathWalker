FROM rockylinux:9 as builder

# Install dependencies
RUN dnf upgrade -y && \
    dnf install -y 'dnf-command(config-manager)' && dnf config-manager --set-enabled crb && dnf install -y epel-release &&  \
    dnf install -y gcc gcc-c++ hwloc-devel hwloc-libs cmake git-core pkgconfig make ninja-build ocl-icd ocl-icd-devel patch findutils wget &&  \
    dnf install -y llvm-devel clang-devel

# Install Rustc and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Build pocl and path_walker
RUN cd /tmp ; git clone https://github.com/pocl/pocl.git && cd /tmp/pocl && \
    cd /tmp/pocl && mkdir b && cd b && cmake -DCMAKE_INSTALL_PREFIX=/tmp/ocl_runtime -DWITH_LLVM_CONFIG=/usr/bin/llvm-config .. && \
    make -j4 && make install

# Copy pocl source
COPY . /tmp/path_walker

# Build path_walker
RUN source "$HOME/.cargo/env" && cd /tmp/path_walker &&  \
    cargo build --release && mkdir /app && mv /tmp/path_walker/target/release/path_walker /app/path_walker

FROM redhat/ubi9-minimal as runtime

# Install dependencies
RUN microdnf upgrade -y && \
    microdnf install -y wget && \
    wget -O /tmp/epel-release-latest-9.noarch.rpm https://dl.fedoraproject.org/pub/epel/epel-release-latest-9.noarch.rpm && \
    wget -O /tmp/ocl-icd-2.2.13-4.el9.x86_64.rpm https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/o/ocl-icd-2.2.13-4.el9.x86_64.rpm && \
    rpm -i /tmp/epel-release-latest-9.noarch.rpm /tmp/ocl-icd-2.2.13-4.el9.x86_64.rpm && \
    microdnf install -y clinfo && microdnf remove -y wget

# Copy pocl and path_walker
COPY --from=builder /app                    /app
COPY --from=builder /tmp/ocl_runtime        /usr
COPY --from=builder /tmp/ocl_runtime/etc    /etc

# Check if pocl is installed correctly
RUN sed -i 's#/tmp/ocl_runtime/lib64/libpocl.so.2.11.0#/usr/lib64/libpocl.so#g' /etc/OpenCL/vendors/pocl.icd && \
    sed -i 's#/tmp/ocl_runtime#/usr#g'                                          /usr/lib64/pkgconfig/pocl.pc

# Set working directory
WORKDIR /app

# Expose port for REST API Communication
EXPOSE 8080

# Run path_walker
CMD /app/path_walker
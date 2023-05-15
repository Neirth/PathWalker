FROM redhat/ubi9:latest as builder

# Install dependencies
RUN dnf upgrade -y && \
    dnf install -y https://dl.fedoraproject.org/pub/epel/epel-release-latest-9.noarch.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/o/ocl-icd-2.2.13-4.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/o/ocl-icd-devel-2.2.13-4.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-2.4.1-5.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-libs-2.4.1-5.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/o/opencl-headers-3.0-6.20201007gitd65bcc5.el9.noarch.rpm && \
    dnf install -y llvm-devel clang-devel gcc gcc-c++ cmake git-core pkgconfig make patch findutils wget

# Install Rustc and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Copy project source
COPY . /tmp/path_walker

# Build path_walker
RUN source "$HOME/.cargo/env" && cd /tmp/path_walker && \
    cargo build --release && mkdir /app && mv /tmp/path_walker/target/release/path_walker /app/path_walker

FROM redhat/ubi9:latest as runtime

# Copy pocl and path_walker
COPY --from=builder /app    /app

# Install dependencies
RUN dnf upgrade -y && \
    dnf install -y https://dl.fedoraproject.org/pub/epel/epel-release-latest-9.noarch.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/o/ocl-icd-2.2.13-4.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-2.4.1-5.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-libs-2.4.1-5.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/c/clang-libs-14.0.6-4.el9_1.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/c/clang-resource-filesystem-14.0.6-4.el9_1.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/l/llvm-libs-14.0.6-1.el9.x86_64.rpm && \
    dnf install -y pocl && rm -Rf /tmp/* && dnf clean all && rm -Rf /var/cache/dnf

# Set working directory
WORKDIR /app

# Expose port for REST API Communication
EXPOSE 8080

# Run path_walker
CMD /app/path_walker
FROM redhat/ubi9:latest as builder

# Install dependencies
RUN dnf upgrade -y && \
    dnf install -y wget && \
    wget -O /tmp/epel-release-latest-9.noarch.rpm https://dl.fedoraproject.org/pub/epel/epel-release-latest-9.noarch.rpm && \
    wget -O /tmp/ocl-icd-2.2.13-4.el9.x86_64.rpm https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/o/ocl-icd-2.2.13-4.el9.x86_64.rpm && \
    wget -O /tmp/ocl-icd-devel-2.2.13-4.el9.x86_64.rpm https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/o/ocl-icd-devel-2.2.13-4.el9.x86_64.rpm && \
    wget -O /tmp/hwloc-2.4.1-5.el9.x86_64.rpm https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-2.4.1-5.el9.x86_64.rpm && \
    wget -O /tmp/hwloc-libs-2.4.1-5.el9.x86_64.rpm https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-libs-2.4.1-5.el9.x86_64.rpm && \
    wget -O /tmp/opencl-headers-3.0-6.20201007gitd65bcc5.el9.noarch.rpm https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/o/opencl-headers-3.0-6.20201007gitd65bcc5.el9.noarch.rpm && \
    dnf install -y gcc gcc-c++ cmake git-core pkgconfig make patch findutils wget &&  \
    dnf install -y llvm-devel clang-devel /tmp/epel-release-latest-9.noarch.rpm /tmp/ocl-icd-2.2.13-4.el9.x86_64.rpm /tmp/ocl-icd-devel-2.2.13-4.el9.x86_64.rpm \
                   /tmp/hwloc-2.4.1-5.el9.x86_64.rpm /tmp/hwloc-libs-2.4.1-5.el9.x86_64.rpm /tmp/opencl-headers-3.0-6.20201007gitd65bcc5.el9.noarch.rpm

# Install RustC and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Copy project source
COPY . /tmp/path_walker

# Build path_walker
RUN source "$HOME/.cargo/env" && cd /tmp/path_walker &&  \
    cargo build --release && mkdir /app && mv /tmp/path_walker/target/release/path_walker /app/path_walker

FROM redhat/ubi9:latest as runtime


# Copy pocl and path_walker
COPY --from=builder /app                    /app

RUN dnf upgrade -y && \
    dnf install -y https://dl.fedoraproject.org/pub/epel/epel-release-latest-9.noarch.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/o/ocl-icd-2.2.13-4.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-2.4.1-5.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/devel/x86_64/os/Packages/h/hwloc-libs-2.4.1-5.el9.x86_64.rpm \
                   https://dl.rockylinux.org/pub/rocky/9/AppStream/x86_64/os/Packages/l/llvm-libs-14.0.6-1.el9.x86_64.rpm && \
    dnf install -y pocl && rm -Rf /tmp/*rpm


# Set working directory
WORKDIR /app

# Expose port for REST API Communication
EXPOSE 8080

# Run path_walker
CMD /app/path_walker
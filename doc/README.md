# yymicros Documentation

## Synopsis

ArceOS is a modular Unikernel Operating system. Is solves the following problems:

1. **Performance**: Providing applications are trustworthy, not only is there no use to isolate the user space so as to make it only trap into privilege mode to reach the system resource, but we can also remove any address space mapping and the majority of components that seems reductant in single-service application. Although it sacrifices functional variety in some extent, it achieves great boosts in both bootstrap and scheduling.
2. **Security**: It is designed using the security-oriented Rust programming language. Also it's tiny enough to make it isolated to any other of applications along with the OS.
3. **Ecosystem**: It is primarily supported the most of Linux system call, the C language library and standard libraries. Also, it is able to be easily extended to a monolithic or micro kernel for specific use.
4. **Modularity**: Fully customizability is the future tendency in the development of operating systems. 

Based on this, we designed this yymicros kernel and its infrastructures.


#!/bin/bash

# Create the output directory if it doesn't exist
output_dir="elf"
src_dir="src"

mkdir -p $output_dir

# Loop over all .c files in the src directory
for src_file in $src_dir/*.c; do
    # Get the filename without the directory and the extension
    filename=$(basename -- "$src_file")
    filename_without_ext="${filename%.*}"
    
    # Compile each .c file to an .elf file in the elf directory
    riscv64-unknown-elf-gcc -o "$output_dir/$filename_without_ext.elf" "$src_file" -nostdlib -static
    
    # Check if the compilation was successful
    if [ $? -eq 0 ]; then
        echo "Compiled $src_file to $output_dir/$filename_without_ext.elf"
    else
        echo "Failed to compile $src_file"
    fi
done

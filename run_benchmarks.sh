#!/usr/bin/env bash
# VHP vs PHP Performance Benchmark Runner
# This is a simple wrapper that calls the Python script

# Check if Python 3 is available
if command -v python3 &> /dev/null; then
    exec python3 run_benchmarks.py "$@"
elif command -v python &> /dev/null; then
    exec python run_benchmarks.py "$@"
else
    echo "Error: Python 3 is required to run benchmarks"
    echo "Please install Python 3 or use run_benchmarks.py directly"
    exit 1
fi

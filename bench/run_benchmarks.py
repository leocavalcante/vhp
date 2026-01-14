#!/usr/bin/env python3
"""VHP vs PHP Performance Benchmark Runner"""

import os
import sys
import subprocess
import time
import statistics
from pathlib import Path

# Colors
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    BOLD = '\033[1m'
    NC = '\033[0m'

# Configuration
VHP_BIN = os.environ.get('VHP_BIN', './target/release/vhp')
PHP_BIN = os.environ.get('PHP_BIN', 'php')
BENCH_DIR = 'bench'
ITERATIONS = int(os.environ.get('BENCH_ITERATIONS', '5'))

def check_binaries():
    """Check if required binaries exist"""
    if not os.path.isfile(VHP_BIN):
        print(f"{Colors.RED}Error: VHP binary not found at {VHP_BIN}{Colors.NC}")
        print("Please build VHP first: make release")
        sys.exit(1)

    try:
        subprocess.run([PHP_BIN, '-v'], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print(f"{Colors.RED}Error: PHP binary not found{Colors.NC}")
        print("Please install PHP or set PHP_BIN environment variable")
        sys.exit(1)

def get_php_version():
    """Get PHP version"""
    result = subprocess.run([PHP_BIN, '-v'], capture_output=True, text=True)
    return result.stdout.split()[1]

def get_vhp_version():
    """Get VHP version"""
    return "0.1.0 (dev)"

def run_benchmark(bench_file, binary):
    """Run a single benchmark and measure time"""
    start = time.perf_counter()
    subprocess.run(
        [binary, os.path.join(BENCH_DIR, bench_file)],
        capture_output=True,
        check=False
    )
    end = time.perf_counter()
    return (end - start) * 1000  # Convert to milliseconds

def benchmark_file(bench_file):
    """Run benchmark multiple times and calculate average"""
    bench_name = os.path.splitext(bench_file)[0]

    print(f"{Colors.BOLD}Running: {bench_name}{Colors.NC}")

    # Run VHP benchmarks
    print("  VHP: ", end='', flush=True)
    vhp_times = []
    for _ in range(ITERATIONS):
        time_ms = run_benchmark(bench_file, VHP_BIN)
        vhp_times.append(time_ms)
        print(".", end='', flush=True)
    print()

    # Run PHP benchmarks
    print("  PHP: ", end='', flush=True)
    php_times = []
    for _ in range(ITERATIONS):
        time_ms = run_benchmark(bench_file, PHP_BIN)
        php_times.append(time_ms)
        print(".", end='', flush=True)
    print()

    # Calculate averages
    vhp_avg = statistics.mean(vhp_times)
    php_avg = statistics.mean(php_times)

    # Calculate speedup
    speedup = php_avg / vhp_avg if vhp_avg > 0 else 1.0

    # Determine if VHP is faster or slower
    if speedup >= 1.0:
        comparison = f"{Colors.GREEN}{speedup:.2f}x faster{Colors.NC}"
    else:
        slowdown = vhp_avg / php_avg
        comparison = f"{Colors.RED}{slowdown:.2f}x slower{Colors.NC}"

    # Print results
    print(f"  {Colors.BLUE}VHP avg:{Colors.NC} {vhp_avg:.2f} ms")
    print(f"  {Colors.BLUE}PHP avg:{Colors.NC} {php_avg:.2f} ms")
    print(f"  {Colors.YELLOW}Result:{Colors.NC} VHP is {comparison}")
    print()

    return {
        'name': bench_name,
        'vhp': vhp_avg,
        'php': php_avg,
        'speedup': speedup
    }

def print_summary(results):
    """Print summary table"""
    print()
    print(f"{Colors.BOLD}========================================{Colors.NC}")
    print(f"{Colors.BOLD}         BENCHMARK SUMMARY{Colors.NC}")
    print(f"{Colors.BOLD}========================================{Colors.NC}")
    print()

    print(f"{'Benchmark':<25} {'VHP (ms)':>12} {'PHP (ms)':>12} {'Speedup':>12}")
    print(f"{'-'*25} {'-'*12} {'-'*12} {'-'*12}")

    for result in results:
        speedup = result['speedup']
        if speedup >= 1.0:
            speedup_str = f"{Colors.GREEN}{speedup:.2f}x{Colors.NC}"
        else:
            speedup_str = f"{Colors.RED}{speedup:.2f}x{Colors.NC}"

        print(f"{result['name']:<25} {result['vhp']:>12.2f} {result['php']:>12.2f} {speedup_str:>20}")

    print()
    print(f"{'-'*25} {'-'*12} {'-'*12} {'-'*12}")

    # Calculate averages
    avg_vhp = statistics.mean([r['vhp'] for r in results])
    avg_php = statistics.mean([r['php'] for r in results])
    overall_speedup = avg_php / avg_vhp

    if overall_speedup >= 1.0:
        overall_str = f"{Colors.GREEN}{overall_speedup:.2f}x{Colors.NC}"
    else:
        overall_str = f"{Colors.RED}{overall_speedup:.2f}x{Colors.NC}"

    print(f"{'AVERAGE':<25} {avg_vhp:>12.2f} {avg_php:>12.2f} {overall_str:>20}")
    print()

def main():
    """Main execution"""
    print(f"{Colors.BOLD}VHP vs PHP Performance Benchmark{Colors.NC}")
    print(f"{Colors.BOLD}================================={Colors.NC}")
    print()

    check_binaries()

    php_version = get_php_version()
    vhp_version = get_vhp_version()

    print(f"VHP Version: {Colors.BLUE}{vhp_version}{Colors.NC}")
    print(f"PHP Version: {Colors.BLUE}{php_version}{Colors.NC}")
    print(f"Iterations:  {Colors.BLUE}{ITERATIONS}{Colors.NC}")
    print()
    print(f"{Colors.BOLD}========================================{Colors.NC}")
    print()

    # Find all benchmark files
    bench_files = sorted(Path(BENCH_DIR).glob('*.php'))

    if not bench_files:
        print(f"{Colors.RED}No benchmark files found in {BENCH_DIR}{Colors.NC}")
        sys.exit(1)

    # Run benchmarks and collect results
    results = []
    for bench_file in bench_files:
        result = benchmark_file(bench_file.name)
        results.append(result)

    # Print summary
    print_summary(results)

    print(f"{Colors.GREEN}Benchmark complete!{Colors.NC}")

if __name__ == '__main__':
    main()

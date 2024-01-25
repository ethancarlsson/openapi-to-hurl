#!/bin/zsh

{
	hyperfine --warmup 50 'target/release/openapi-to-hurl test_files/pet_store_advanced.json --output-to console' -u millisecond --shell=none &&
 	git log --format="%H" -n 1 &&
	echo v0.1.0;
} | python3 format_benchmarks.py >> bench_over_time.csv


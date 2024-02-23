#!/bin/zsh
{
	hyperfine --warmup 50 'target/release/openapi-to-hurl test_files/pet_store_advanced.json' -u millisecond --shell=none &&
 	git log --format="%H" -n 1 &&
	target/release/openapi-to-hurl -v
} | python3 format_benchmarks.py >> bench_over_time.csv

{
	schema=$(cat test_files/pet_store_advanced.json)

	hyperfine --warmup 50 "echo $schema | target/release/openapi-to-hurl" -u millisecond --shell=none &&
 	git log --format="%H" -n 1 &&
	target/release/openapi-to-hurl -v
} | python3 format_benchmarks.py >> bench_over_time_stdin.csv

{
	mkdir test_hurl_files;

	hyperfine --warmup 50 'target/release/openapi-to-hurl test_files/pet_store_advanced.json -o test_hurl_files' -u millisecond --shell=none &&
 	git log --format="%H" -n 1 &&
	target/release/openapi-to-hurl -v
} | python3 format_benchmarks.py >> bench_over_time_files.csv

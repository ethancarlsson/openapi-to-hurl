const { exec } = require("child_process");

exports.preTagGeneration = function (tag) {
  exec(
    `
	{
		hyperfine --warmup 50 '../target/release/openapi-to-hurl ../test_files/pet_store_advanced.json --output-to console' -u millisecond --shell=none &&
		git log --format="%H" -n 1 &&
		echo ${tag};
	} | python3 format_benchmarks.py >> bench_over_time.csv
	`,
    (error, stdout, stderr) => console.log(`exec_result: err: ${error}, output: ${stdout}, stderr ${stderr}`),
  );
};

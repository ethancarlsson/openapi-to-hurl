const util = require("util");
const exec = util.promisify(require("child_process").exec);

exports.preTagGeneration = async function (tag) {
  // Run the command a couple times to warm it up
  for (let i = 0; i < 5; i++) {
    try {
      await exec(
        "/home/runner/work/openapi-to-hurl/openapi-to-hurl/target/release/openapi-to-hurl /home/runner/work/openapi-to-hurl/openapi-to-hurl/test_files/pet_store_advanced.json",
      );
    } catch (e) {
      console.error(e);
    }
  }

  try {
    const { stderr, stdout } = await exec(
      `
	{
		hyperfine --warmup 50 '/home/runner/work/openapi-to-hurl/openapi-to-hurl/target/release/openapi-to-hurl /home/runner/work/openapi-to-hurl/openapi-to-hurl/test_files/pet_store_advanced.json' -u millisecond --shell=none &&
		git log --format="%H" -n 1 &&
		echo ${tag};
	} | python3 format_benchmarks.py >> bench_over_time.csv

	{
		schema=$(cat test_files/pet_store_advanced.json)

		hyperfine --warmup 50 "echo $schema | /home/runner/work/openapi-to-hurl/openapi-to-hurl/target/release/openapi-to-hurl" -u millisecond --shell=none &&
		git log --format="%H" -n 1 &&
		echo ${tag};
	} | python3 format_benchmarks.py >> bench_over_time_stdin.csv

	{
		mkdir /test_hurl_files;

		hyperfine --warmup 50 '/home/runner/work/openapi-to-hurl/openapi-to-hurl/target/release/openapi-to-hurl /home/runner/work/openapi-to-hurl/openapi-to-hurl/test_files/pet_store_advanced.json -o /test_hurl_files' -u millisecond --shell=none &&
		git log --format="%H" -n 1 &&
		echo ${tag};
	} | python3 format_benchmarks.py >> bench_over_time_files.csv
	`,
    );
    console.log(`output: ${stdout}, stderr ${stderr}`);
  } catch (e) {
    console.error(`err: ${e}`);
  }
};


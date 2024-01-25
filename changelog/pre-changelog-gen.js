const util = require("util");
const exec = util.promisify(require("child_process").exec);

exports.preTagGeneration = async function (tag) {
  // Run the command a couple times to warm it up
  for (let i = 0; i < 5; i++) {
    try {
      await exec(
        "/home/runner/work/openapi-to-hurl/openapi-to-hurl/target/release/openapi-to-hurl /home/runner/work/openapi-to-hurl/openapi-to-hurl/test_files/pet_store_advanced.json --output-to console' -u millisecond --shell=none",
      );
    } catch (e) {
      console.error(e);
    }
  }

  try {
    const { stderr, stdout } = await exec(
      `
	{
		hyperfine --warmup 50 '/home/runner/work/openapi-to-hurl/openapi-to-hurl/target/release/openapi-to-hurl /home/runner/work/openapi-to-hurl/openapi-to-hurl/test_files/pet_store_advanced.json --output-to console' -u millisecond --shell=none &&
		git log --format="%H" -n 1 &&
		echo ${tag};
	} | python3 format_benchmarks.py >> bench_over_time.csv
	`,
    );
    console.log(`output: ${stdout}, stderr ${stderr}`);
  } catch (e) {
    console.error(`err: ${e}`);
  }
};

exports.preTagGeneration("test");

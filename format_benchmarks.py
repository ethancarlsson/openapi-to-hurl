import sys
import re

# This is a single use script. Input is taken from this command:
# {
#    hyperfine --warmup 10 'target/release/openapi-to-hurl test_files/pet_store_advanced.json --output-to console' -u millisecond
#    && git log --format="%H" -n 1
#    && echo vx.x.x;
# }
# Intended output is a csv file

result = []
for i, line in enumerate(sys.stdin):
    # Reading a line like this:
    # Time (mean ± σ):       7.7 ms ±   0.2 ms    [User: 6.6 ms, System: 0.6 ms]
    if i == 1:
        result.append(re.search("\d+\.\d+", line).group(0))
    # Reading a line like this:
    # Range (min … max):     7.1 ms …  30.4 ms    297 runs
    if i == 2:
        ranges_and_runs = re.findall("\d+\.\d+", line)
        runs = re.search("\d+ runs", line).group(0)
        ranges_and_runs.append(runs[:len(runs) - 5])
        result += ranges_and_runs

    # Reading the commit ID
    if i == 4:
        result.append(line.strip())
    # Reading the version number passed in from pre-changelog file
    if i == 5:
        result.append(line)



sys.stdout.write(",".join(result))

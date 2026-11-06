class NamesAsWrittenTest < AcceptanceTest
  def setup = workspace("atelier")

  test "outcomes named 200 and 404, which YAML alone would read as numbers, are names as written: validate finds nothing" do
    write_file ".roostr/processes/show-booking.yml", fixture("processes/show-booking.yml") # outcomes 200 and 404

    validate

    assert_success
    assert_stdout "No problems found"
  end

  test "show prints the outcomes 200 and 404 as written" do
    write_file ".roostr/processes/show-booking.yml", fixture("processes/show-booking.yml")

    show "show-booking"

    assert_success
    assert_stdout "  200   the booking exists: its details are returned\n"
    assert_stdout "  404   there is no booking with that id: nothing is returned\n"
  end

  test "show --json names the outcomes with the strings \"200\" and \"404\"" do
    write_file ".roostr/processes/show-booking.yml", fixture("processes/show-booking.yml")

    show "show-booking", "--json"

    assert_success
    assert_json "process.outcomes.*.name", ["200", "404"]
  end

  test "operations named 1 and 2 and an environment named 8080 are names as written: the overview prints them, exit 0" do
    write_file ".roostr/project.yml", fixture("project/numbered-operations.yml") # operations 1: bin/setup and 2: bin/dev; environments 8080 and production

    overview

    assert_success
    assert_stdout(/^  1 +bin\/setup\n  2 +bin\/dev\n/)
    assert_stdout(/^  8080 +localhost:8080, the app without the proxy\n/)
  end

  test "roostr --json names the operations \"1\" and \"2\" and the environment \"8080\", strings" do
    write_file ".roostr/project.yml", fixture("project/numbered-operations.yml")

    overview "--json"

    assert_success
    assert_json "project.operations.*.name", ["1", "2"]
    assert_json "project.environments.*.name", ["8080", "production"]
  end
end

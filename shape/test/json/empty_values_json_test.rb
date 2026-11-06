class EmptyValuesJsonTest < AcceptanceTest
  def setup = workspace("atelier")

  test "in a record, group and try with nothing after them are null, as if they were not written" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops-left-empty.yml") # group:, entrypoints:, try:, outcomes:, why:, checked: and a files note: with nothing after them

    show "import-workshops", "--json"

    assert_success
    assert_json "process.group", nil
    assert_json "process.try", nil
  end

  test "checked with nothing after it is null" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops-left-empty.yml")

    show "import-workshops", "--json"

    assert_json "process.checked", nil
  end

  test "entrypoints, outcomes and why with nothing after them are empty lists" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops-left-empty.yml")

    show "import-workshops", "--json"

    assert_json "process.entrypoints", []
    assert_json "process.outcomes", []
    assert_json "process.why", []
  end

  test "a files item whose note has nothing after it has note null" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops-left-empty.yml")

    show "import-workshops", "--json"

    assert_json "process.files", [{ "path" => "app/models/booking.rb", "note" => nil }]
  end

  test "in project.yml, stack with nothing after it and description written as an empty string are both null" do
    write_file ".roostr/project.yml", fixture("project/keys-left-empty.yml") # stack:, watch_out:, operations:, environments: with nothing after them, description: ""

    overview "--json"

    assert_success
    assert_json "project.stack", nil
    assert_json "project.description", nil
  end

  test "watch_out, operations and environments with nothing after them are empty lists" do
    write_file ".roostr/project.yml", fixture("project/keys-left-empty.yml")

    overview "--json"

    assert_json "project.watch_out", []
    assert_json "project.operations", []
    assert_json "project.environments", []
  end

  test "an operation with no command is left out of operations" do
    write_file ".roostr/project.yml", fixture("project/deploy-without-command.yml") # start, test, and deploy: with its command line still commented out

    overview "--json"

    assert_success
    assert_json "project.operations.*.name", ["start", "test"]
  end

  test "an environment with nothing after its name is left out of environments" do
    write_file ".roostr/project.yml", fixture("project/staging-left-empty.yml") # staging: with nothing after it, then production

    overview "--json"

    assert_success
    assert_json "project.environments.*.name", ["production"]
  end
end

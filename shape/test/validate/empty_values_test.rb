class EmptyValuesTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes, 7 entrypoints, every listed file present

  test "a record whose optional keys all have nothing after them is no finding: each counts as not written" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops-left-empty.yml") # group:, entrypoints:, try:, outcomes:, why:, checked: and a files note: with nothing after them

    validate

    assert_success
    assert_stdout "No problems found"
  end

  test "project.yml whose optional keys have nothing after them, or an empty string, is no finding" do
    write_file ".roostr/project.yml", fixture("project/keys-left-empty.yml") # stack:, watch_out:, operations:, environments: with nothing after them, description: ""

    validate

    assert_success
    assert_stdout "No problems found"
  end

  test "an operation with no command, deploy: with its command line still commented out, is a finding on operations.deploy" do
    write_file ".roostr/project.yml", fixture("project/deploy-without-command.yml")

    validate

    assert_exit 1
    assert_stdout ".roostr/project.yml: operations.deploy: "
  end

  test "an environment with nothing after its name is a finding on environments.staging" do
    write_file ".roostr/project.yml", fixture("project/staging-left-empty.yml")

    validate

    assert_exit 1
    assert_stdout ".roostr/project.yml: environments.staging: "
  end

  test "a description with nothing after it is a missing description, which is still a finding" do
    write_file ".roostr/processes/export-bookings.yml", "name: Export bookings\ndescription:\n"

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/export-bookings.yml: description: "
  end
end

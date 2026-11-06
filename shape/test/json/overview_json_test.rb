class OverviewJsonTest < AcceptanceTest
  def setup = workspace("atelier") # not under Git; three processes, seven entrypoints, two operations without a note and one with

  test "roostr --json gives the project, every process, the entrypoints and what needs attention, exactly as laid out, and writes nothing" do
    overview "--json"

    assert_success
    assert_json_is expected("overview_atelier.json")
    assert_nothing_written
  end

  test "a project.yml with only a name gives stack and description null, and watch_out, operations and environments empty" do
    write_file ".roostr/project.yml", "name: Atelier\n"

    overview "--json"

    assert_json "project", { "name" => "Atelier", "stack" => nil, "description" => nil,
                             "watch_out" => [], "operations" => [], "environments" => [] }
  end

  test "processes holds every readable record, in id order" do
    write_file ".roostr/processes/import-workshops.yml", fixture("processes/import-workshops.yml")
    write_file ".roostr/processes/fix-double-booking.yml", fixture("processes/fix-double-booking.yml")

    overview "--json"

    assert_json "processes.*.id", ["book-workshop", "cancel-booking", "fix-double-booking", "import-workshops", "remind-attendees"]
  end

  test "an operation note of 161 characters, over the limit validate reports, still gives the overview as JSON, with the note whole" do
    write_file ".roostr/project.yml", fixture("project/note-161.yml") # the note of the operation test has 161 characters

    overview "--json"

    assert_success
    assert_json "project.operations.2.note", "asks for the production vault password, which is in the shared password manager under Atelier, " \
                                             "then runs the open migrations and restarts the workers, one by one"
  end
end

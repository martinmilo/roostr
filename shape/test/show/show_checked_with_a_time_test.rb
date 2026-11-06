class ShowCheckedWithATimeTest < AcceptanceTest
  def setup = workspace("atelier")

  test "a checked date with a time is shown as written, on the line labelled checked" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked-at-a-time.yml") # checked: 2026-10-08 14:30

    show "remind-attendees"

    assert_success
    assert_stdout(/^  checked  2026-10-08 14:30$/)
  end

  test "in show --json, a checked date with a time is the string as written" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked-at-a-time.yml")

    show "remind-attendees", "--json"

    assert_success
    assert_json "process.checked", "2026-10-08 14:30"
  end
end

class DuplicateKeyTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes; project.yml has stack Rails 8, PostgreSQL, Solid Queue

  test "a key written twice in a record, files on line 4 and again on line 8, is a finding on files that names both lines, exit 1" do
    write_file ".roostr/processes/cancel-booking.yml", fixture("processes/cancel-booking-files-twice.yml")

    validate

    assert_exit 1
    assert_stdout(/^\.roostr\/processes\/cancel-booking\.yml: files: .*\b4\b.*\b8\b/)
  end

  test "a key written twice inside an outcome is a finding on its dotted path, outcomes.refunded.when" do
    write_file ".roostr/processes/cancel-booking.yml", fixture("processes/cancel-booking-when-twice.yml") # when: on lines 10 and 12, in the outcome refunded

    validate

    assert_exit 1
    assert_stdout ".roostr/processes/cancel-booking.yml: outcomes.refunded.when: "
  end

  test "a key written twice in project.yml, stack on line 2 and again on line 6, is a finding that names both lines" do
    write_file ".roostr/project.yml", fixture("project/stack-twice.yml")

    validate

    assert_exit 1
    assert_stdout(/^\.roostr\/project\.yml: stack: .*\b2\b.*\b6\b/)
  end

  test "a record with a key written twice is still readable, and the value written last counts: checked 2026-09-01, then checked 2026-10-08 appended, is 2026-10-08" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked-twice.yml")

    show "remind-attendees", "--json"

    assert_success
    assert_json "process.checked", "2026-10-08"
  end

  test "project.yml with a key written twice is still readable: the overview prints, exit 0, with the stack written last" do
    write_file ".roostr/project.yml", fixture("project/stack-twice.yml") # stack: Rails 7, PostgreSQL, then stack: Rails 8, PostgreSQL, Solid Queue

    overview

    assert_success
    assert_stdout(/\AAtelier +Rails 8, PostgreSQL, Solid Queue\n/)
  end
end

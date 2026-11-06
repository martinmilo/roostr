class CheckValidateTest < AcceptanceTest
  def setup = atelier_under_git # cancel-booking lists http DELETE /bookings/:id; nothing uncommitted, no scan script

  test "check reports what validate finds: a record that lists an entrypoint no longer in entrypoints.tsv is named with the entrypoint, exit 1" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/cancel-route-dropped.tsv") # rescanned after the cancel route was removed

    check

    assert_exit 1
    assert_stdout "cancel-booking.yml"
    assert_stdout "http DELETE /bookings/:id"
  end

  test "validate's findings come first, under the heading VALIDATE, with a paragraph saying to correct each in the file it names, then the other blocks" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/cancel-route-dropped.tsv")
    edit_file "app/jobs/booking_reminder_job.rb" # listed by remind-attendees

    check

    assert_stdout(/\AVALIDATE\n  Correct each of these in the file it names\. roostr validate lists the same\n  problems\.\n    \.roostr\/processes\/cancel-booking\.yml: .*\n\nPROCESSES\n/)
  end

  test "with --base HEAD and nothing changed since the last commit, a validate finding committed earlier is not reported: No problems found, exit 0" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/cancel-route-dropped.tsv")
    commit "Drop the cancel route", on: "2026-10-06"

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "with --base HEAD and a code file changed, a validate finding in a record the work did not touch is reported" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/cancel-route-dropped.tsv")
    commit "Drop the cancel route", on: "2026-10-06"
    edit_file "app/controllers/workshops_controller.rb" # listed by no record

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "cancel-booking.yml"
    assert_stdout "http DELETE /bookings/:id"
  end

  test "with --base HEAD and only a record changed, an old validate finding in another record is reported and the scan script is not run" do
    commit_scan_script "leaves-a-trace.sh" # creates scan-was-run whenever it runs
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/cancel-route-dropped.tsv")
    commit "Drop the cancel route", on: "2026-10-06"
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml") # the same record with checked: 2026-10-08

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "cancel-booking.yml"
    refute_file "scan-was-run"
  end

  test "check --base HEAD lists under VALIDATE a record not committed yet whose description holds U+202E, a character that changes how text shows: exit 1" do
    write_file ".roostr/processes/refund-booking.yml", "name: Refund a booking\ndescription: \"Refund a\\u202E cancelled booking\"\n"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout(/\AVALIDATE\n/)
    assert_stdout "U+202E"
  end
end

class CheckStagedTest < AcceptanceTest
  def setup = atelier_under_git # committed once, no scan script; remind-attendees lists app/jobs/booking_reminder_job.rb

  test "a new record left untracked, as git commit -am leaves it, is listed under NOT STAGED with the remedy to git add it, exit 1" do
    the_waiting_list_added_with_its_record
    stage_everything_but ".roostr/processes/join-waiting-list.yml"

    check "--staged"

    assert_exit 1
    assert_stdout "NOT STAGED\n"
    assert_stdout "\n    .roostr/processes/join-waiting-list.yml\n"
    assert_stdout "git add <path>"
  end

  test "the same untracked record does not hold back check --base HEAD, which judges the working tree as it stands" do
    the_waiting_list_added_with_its_record
    stage_everything_but ".roostr/processes/join-waiting-list.yml"

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a listed file's change staged and its record's edit not: the record is listed under NOT STAGED, and its process is not also listed under PROCESSES" do
    edit_file "app/jobs/booking_reminder_job.rb"
    git "add", "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml"

    check "--staged"

    assert_exit 1
    assert_stdout "NOT STAGED\n"
    assert_stdout "\n    .roostr/processes/remind-attendees.yml\n"
    refute_stdout "PROCESSES"
  end

  test "a record whose edit was staged and that was then edited again is listed under NOT STAGED" do
    edit_file ".roostr/processes/remind-attendees.yml"
    git "add", ".roostr/processes/remind-attendees.yml"
    edit_file ".roostr/processes/remind-attendees.yml"

    check "--staged"

    assert_exit 1
    assert_stdout "\n    .roostr/processes/remind-attendees.yml\n"
  end

  test "a scan script deleted without staging the deletion is listed under NOT STAGED as .roostr/scan" do
    commit_scan_prints fixture("scans/atelier.txt")
    delete_file ".roostr/scan"

    check "--staged"

    assert_exit 1
    assert_stdout "NOT STAGED\n"
    assert_stdout "\n    .roostr/scan\n"
  end

  test "entrypoints.tsv and project.yml changed and not staged are both listed, in path order" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv")
    edit_file ".roostr/project.yml"

    check "--staged"

    assert_exit 1
    assert_stdout "\n    .roostr/entrypoints.tsv\n    .roostr/project.yml\n"
  end

  test "a record Git ignores and the lock Emacs keeps beside a record are not listed: No problems found, exit 0" do
    write_file ".gitignore", ".roostr/processes/draft-*.yml\n"
    commit "Keep drafts of records out of Git"
    write_file ".roostr/processes/draft-fix-double-booking.yml", fixture("processes/fix-double-booking.yml")
    emacs_is_editing "cancel-booking"

    check "--staged"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "an untracked file under .roostr/ that roostr does not read, .roostr/NOTES.md, is not listed" do
    write_file ".roostr/NOTES.md", "Describe the waiting list once it has a job of its own.\n"

    check "--staged"

    assert_success
    refute_stdout "NOT STAGED"
  end

  test "NOT STAGED comes first: an untracked record that does not parse is listed under NOT STAGED, then under VALIDATE" do
    write_file ".roostr/processes/fix-double-booking.yml", "name: [Fix a double booking\n"

    check "--staged"

    assert_exit 1
    assert_stdout(/\ANOT STAGED\n.*\n\nVALIDATE\n/m)
  end

  test "a change to a listed file that is staged counts although the working tree no longer has it: PROCESSES lists it" do
    stage_a_change_then_undo_it "app/jobs/booking_reminder_job.rb"

    check "--staged"

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end
end

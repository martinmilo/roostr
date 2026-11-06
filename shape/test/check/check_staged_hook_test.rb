class CheckStagedHookTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb

  test "git commit -a while a new record is untracked is refused: the hook lists the record under NOT STAGED, and no commit is made" do
    pre_commit_hook "roostr check --staged"
    the_waiting_list_added_with_its_record

    git_commit "-a", "-m", "Add the waiting list"

    assert_exit 1
    assert_stderr "NOT STAGED"
    assert_stderr ".roostr/processes/join-waiting-list.yml"
    refute_commit_made
  end

  test "git commit -a when every change is to a tracked file makes the commit: the hook judges the index Git builds for -a, which holds the record's edit" do
    pre_commit_hook "roostr check --staged"
    edit_file "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml" # staged by -a, not by git add

    git_commit "-a", "-m", "Send the reminder two days ahead"

    assert_success
    assert_commit_made
  end

  test "git commit with paths that name a listed file but not its record is refused, although the record's edit was staged: the commit holds only the paths named" do
    pre_commit_hook "roostr check --staged"
    edit_file "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml"
    git "add", ".roostr/processes/remind-attendees.yml"

    git_commit "-m", "Send the reminder two days ahead", "app/jobs/booking_reminder_job.rb"

    assert_exit 1
    assert_stderr ".roostr/processes/remind-attendees.yml"
    refute_commit_made
  end

  test "git commit with paths that name a listed file and its record makes the commit" do
    pre_commit_hook "roostr check --staged"
    edit_file "app/jobs/booking_reminder_job.rb"
    edit_file ".roostr/processes/remind-attendees.yml"

    git_commit "-m", "Send the reminder two days ahead", "app/jobs/booking_reminder_job.rb", ".roostr/processes/remind-attendees.yml"

    assert_success
    assert_commit_made
  end
end

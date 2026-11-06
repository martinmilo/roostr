class SessionSnapshotTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb

  test "with uncommitted work in the tree, an edit, a new file and a deletion, session start still writes only in .git/roostr/sessions/: no Git object, not the index" do
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    write_file "app/jobs/waiting_list_job.rb", "# Placeholder standing in for app/jobs/waiting_list_job.rb in the Atelier fixture.\n"
    delete_file "app/mailers/booking_mailer.rb"

    session_start

    assert_success
    assert_written_only_under ".git/roostr/sessions/"
  end

  test "an untracked file under a clean filter, as Git LFS sets one up: session start runs no filter and writes only in .git/roostr/sessions/" do
    untracked_files_under_a_clean_filter

    session_start

    assert_success
    assert_written_only_under ".git/roostr/sessions/"
  end

  test "an untracked nested Git repository beside the user's edit does not cost the snapshot: the edit, left alone in the session, is not the session's work, No problems found" do
    nested_repository "vendor/kiln-firmware"
    the_user_left_edited "app/jobs/booking_reminder_job.rb"
    session_start

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the user's edit made booking_reminder_job.rb exactly 8 MiB, the most that is read: saved again unchanged in the session, its content is the same, not the session's work" do
    edited_to_a_size_of "app/jobs/booking_reminder_job.rb", 8_388_608 # before the session
    session_start
    saved_again_unchanged "app/jobs/booking_reminder_job.rb"

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "the user's edit made booking_reminder_job.rb 8 MiB and one byte, noted by size and modification time and not read: saved again unchanged in the session, it counts as the session's work" do
    edited_to_a_size_of "app/jobs/booking_reminder_job.rb", 8_388_609 # before the session
    session_start
    saved_again_unchanged "app/jobs/booking_reminder_job.rb"

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end

  test "a git status that does not answer within the snapshot's 3 seconds does not hold session start up: it exits within 6 s and prints the map" do
    session_start env: a_git_whose_status_never_answers

    assert_finished_within 6
    assert_success
    assert_map "roostr project map of Atelier"
  end

  test "a git status that does not answer costs the snapshot, not the session: it is recorded first, and a change committed in it is listed by check --session" do
    session_start env: a_git_whose_status_never_answers
    reminder_job_changed_and_committed

    check_session

    assert_exit 1
    assert_stdout "\n    remind-attendees\n      app/jobs/booking_reminder_job.rb\n"
  end
end

class GitCannotBeReadTest < AcceptanceTest
  def setup = atelier_under_git # committed once; book-workshop and cancel-booking list app/models/booking.rb

  test "with git not on the PATH, the overview still prints everything it reads from the files, leaves out LAST 7 DAYS, and ends with NEEDS ATTENTION saying that Git could not be read and why, exit 0" do
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06" # with Git, LAST 7 DAYS and two processes would show
    git_not_on_the_path

    overview

    assert_success
    assert_stdout_is expected("overview_atelier_git_not_found.txt")
  end

  test "when Git refuses the repository, the overview prints, exit 0, and the note gives the first line of what Git said" do
    git_refuses_the_repository

    overview

    assert_success
    assert_stdout "  Git could not be read, so changes to the code are not shown:\n" \
                  "    fatal: detected dubious ownership in repository at '/workspace/atelier'\n"
    refute_stdout "To add an exception"
  end

  test "with a Git older than 2.25, the overview prints, exit 0, and the note says that roostr needs Git 2.25 or later and that this one is 2.20.0" do
    git_older_than_supported

    overview

    assert_success
    assert_stdout "  Git could not be read, so changes to the code are not shown:\n"
    assert_stdout "2.25 or later"
    assert_stdout "2.20.0"
  end

  test "when Git cannot be read, a record that cannot be read is still listed as needing checking, before the note" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"
    git_not_on_the_path

    overview

    assert_stdout "NEEDS ATTENTION\n" \
                  "  .roostr/processes/cancel-booking.yml: needs checking; run roostr validate\n" \
                  "  Git could not be read, so changes to the code are not shown:\n"
  end
end

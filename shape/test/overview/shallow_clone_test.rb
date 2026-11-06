class ShallowCloneTest < AcceptanceTest
  def setup = atelier_under_git # committed once; book-workshop and cancel-booking list app/models/booking.rb

  test "in a shallow clone the overview cannot tell which file changed after which record: it prints, exit 0, leaves out LAST 7 DAYS and the processes of NEEDS ATTENTION, and NEEDS ATTENTION says Git could not be read because the history is shallow, giving git fetch --unshallow" do
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06" # with full history, LAST 7 DAYS and two processes would show
    shallow_clone

    overview

    assert_success
    refute_stdout "LAST 7 DAYS"
    refute_stdout "changed since this record was updated"
    assert_stdout "  Git could not be read, so changes to the code are not shown:\n    "
    assert_stdout "shallow"
    assert_stdout "git fetch --unshallow"
  end
end

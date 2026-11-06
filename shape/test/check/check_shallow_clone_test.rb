class CheckShallowCloneTest < AcceptanceTest
  test "in a shallow clone check without --base cannot run: exit 2, nothing on stdout, and stderr says the history is shallow and gives both --base and git fetch --unshallow" do
    atelier_under_git
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06"
    shallow_clone

    check

    assert_exit 2
    assert_stdout_is ""
    assert_stderr "shallow"
    assert_stderr "--base"
    assert_stderr "git fetch --unshallow"
  end

  test "check --base HEAD in a clone of depth 1 still runs, since no commit after HEAD can be missing: booking.rb edited and not committed lists book-workshop and cancel-booking, exit 1" do
    atelier_under_git
    shallow_clone
    edit_file "app/models/booking.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "\n    book-workshop\n      app/models/booking.rb\n"
    assert_stdout "\n    cancel-booking\n      app/models/booking.rb\n"
  end

  test "check --base origin/main in a clone of depth 2 that holds every commit after main still runs: the branch's one commit, to booking.rb, lists book-workshop and cancel-booking, exit 1" do
    atelier_after_the_waiting_list_week # main has seven commits; book-workshop and cancel-booking list app/models/booking.rb
    branch "longer-bookings"
    edit_file "app/models/booking.rb"
    commit "Let a booking last a whole weekend", on: "2026-10-10"
    shallow_clone depth: 2

    check "--base", "origin/main"

    assert_exit 1
    assert_stdout "\n    book-workshop\n      app/models/booking.rb\n"
    assert_stdout "\n    cancel-booking\n      app/models/booking.rb\n"
  end

  test "check --base origin/main in a clone of depth 2 that cuts short a branch merged after main, leaving its change to booking.rb outside the clone, cannot run: exit 2, nothing on stdout, and stderr says the history after the base is not all in the clone and gives git fetch --unshallow" do
    atelier_under_git
    a_side_branch_merged_into_waiting_list
    shallow_clone depth: 2

    check "--base", "origin/main"

    assert_exit 2
    assert_stdout_is ""
    assert_stderr "git fetch --unshallow"
  end
end

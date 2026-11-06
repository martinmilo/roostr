class ChangesShallowCloneTest < AcceptanceTest
  test "in a shallow clone changes cannot run: exit 2, nothing on stdout, and stderr says the history is shallow and gives git fetch --unshallow" do
    atelier_after_the_waiting_list_week # six commits after Atelier's first, the last on 2026-10-09
    shallow_clone

    changes

    assert_exit 2
    assert_stdout_is ""
    assert_stderr "shallow"
    assert_stderr "git fetch --unshallow"
  end
end

class HiddenFilesTest < AcceptanceTest
  def setup = atelier_under_git # committed once, nothing uncommitted; the processes are book-workshop, cancel-booking and remind-attendees

  test "the lock Emacs keeps while cancel-booking.yml is open, a symbolic link to nothing named dot hash cancel-booking.yml, is not a record: validate finds nothing" do
    emacs_is_editing "cancel-booking"

    validate

    assert_success
    assert_stdout "No problems found"
  end

  test "with the Emacs lock in .roostr/processes/, the overview lists nothing as needing checking" do
    emacs_is_editing "cancel-booking"

    overview

    assert_success
    refute_stdout ".#cancel-booking"
  end

  test "with the Emacs lock in .roostr/processes/, check --base HEAD, as a pre-commit hook runs it, finds nothing and exits 0" do
    emacs_is_editing "cancel-booking"

    check "--base", "HEAD"

    assert_success
    assert_stdout "No problems found"
  end

  test "a file in .roostr/processes/ whose name starts with a dot is not a record, even when it holds one: .refund-booking.yml is not among the processes" do
    write_file ".roostr/processes/.refund-booking.yml", fixture("processes/refund-booking.yml")

    overview "--json"

    assert_success
    assert_json "processes.*.id", ["book-workshop", "cancel-booking", "remind-attendees"]
  end
end

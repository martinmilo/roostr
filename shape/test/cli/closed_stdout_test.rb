class ClosedStdoutTest < AcceptanceTest
  def setup = atelier_under_git # committed once, nothing uncommitted; book-workshop and cancel-booking list app/models/booking.rb

  test "validate --json with a finding, whose stdout is already closed, stops writing quietly: exit 1 as it would have been, and nothing on stderr" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists http POST /refunds, which is not in entrypoints.tsv

    validate "--json", stdout: :closed

    assert_exit 1
    assert_stderr_is ""
  end

  test "check with a problem, whose stdout is already closed, stops writing quietly: exit 1 as it would have been, and nothing on stderr" do
    edit_file "app/models/booking.rb"

    check stdout: :closed

    assert_exit 1
    assert_stderr_is ""
  end

  test "check with nothing to report, whose stdout is already closed, stops writing quietly: exit 0, and nothing on stderr" do
    check stdout: :closed

    assert_success
    assert_stderr_is ""
  end
end

class CheckJsonSessionTest < AcceptanceTest
  def setup = atelier_under_git # remind-attendees lists app/jobs/booking_reminder_job.rb

  test "check --session --json, with the session recorded, gives base as the full hash of the commit the session started at, session as its id, and staged false" do
    session_start
    reminder_job_changed_and_committed

    check_session "--json"

    assert_exit 1
    assert_json "base", /\A\h{40}\z/
    assert_json "session", "5f0c8e2a-9b1d-4c7e-a3f6-0d2b4e6a8c1f"
    assert_json "staged", false
  end

  test "check --session --json with no record for the session gives base HEAD, the base it judged from, and session as the id it read" do
    check_session "--json"

    assert_success
    assert_json "base", "HEAD"
    assert_json "session", "5f0c8e2a-9b1d-4c7e-a3f6-0d2b4e6a8c1f"
  end

  test "check --base HEAD --json gives session null" do
    check "--base", "HEAD", "--json"

    assert_success
    assert_json "session", nil
  end
end

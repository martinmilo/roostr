class JsonOptionPlaceTest < AcceptanceTest
  def setup = atelier_under_git # committed once on 2026-01-15; today is 2026-10-10

  test "--json may come before the command: roostr --json show book-workshop gives the process as JSON" do
    cli "--json", "show", "book-workshop"

    assert_success
    assert_json "process.id", "book-workshop"
  end

  test "--json may come between the command and its argument: roostr show --json book-workshop gives the process as JSON" do
    show "--json", "book-workshop"

    assert_success
    assert_json "process.id", "book-workshop"
  end

  test "--json is never taken for the id: show --json without an id is a mistake in the command, exit 2, as show without an id is" do
    show "--json"

    assert_exit 2
    assert_stdout_is ""
  end

  test "--json combines with --since: changes --since 3d --json gives since as the first day of the period, 2026-10-07" do
    changes "--since", "3d", "--json"

    assert_success
    assert_json "since", "2026-10-07"
  end

  test "--json combines with --base, also in front of it: check --json --base HEAD gives base as written, HEAD" do
    check "--json", "--base", "HEAD"

    assert_success
    assert_json "base", "HEAD"
  end
end

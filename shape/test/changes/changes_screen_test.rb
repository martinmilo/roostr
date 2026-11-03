class ChangesScreenTest < AcceptanceTest
  def setup = atelier_after_the_waiting_list_week # one commit in September, five from 5 to 9 October; today is 2026-10-10

  test "roostr changes prints the first day of the period, the entrypoints added and removed, each changed process with the subjects of its commits newest first, and the count line, and writes nothing" do
    changes

    assert_success
    assert_stdout_is expected("changes_atelier.txt")
    assert_nothing_written
  end
end

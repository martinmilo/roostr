class OverviewAfterAWeekTest < AcceptanceTest
  def setup = atelier_after_the_waiting_list_week # one commit in September, five from 5 to 9 October; today is 2026-10-10

  test "after ENVIRONMENTS the overview prints LAST 7 DAYS, with the count line of roostr changes and a pointer to it, then NEEDS ATTENTION, with one line for each file changed since the record of a process that lists it, and writes nothing" do
    overview

    assert_success
    assert_stdout_is expected("overview_atelier_after_the_week.txt")
    assert_nothing_written
  end
end

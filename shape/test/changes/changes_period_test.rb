class ChangesPeriodTest < AcceptanceTest
  def setup = atelier_under_git # committed once on 2026-01-15; today is 2026-10-10 unless the test says otherwise

  { [] => "2026-10-03", ["--since", "7d"] => "2026-10-03", ["--since", "3d"] => "2026-10-07", ["--since", "2026-09-21"] => "2026-09-21" }.each do |arguments, first_day|
    test "on 2026-10-10, roostr #{["changes", *arguments].join(" ")} starts its first line with CHANGES SINCE #{first_day}, the first day of the period" do
      changes(*arguments)

      assert_success
      assert_stdout "CHANGES SINCE #{first_day}\n"
    end
  end

  test "the value of --since may follow an equals sign: on 2026-10-10, --since=3d written as one word starts the period on 2026-10-07" do
    changes "--since=3d"

    assert_success
    assert_stdout "CHANGES SINCE 2026-10-07\n"
  end

  test "ROOSTR_TODAY sets what today is: on 2026-11-02 the last 7 days start on 2026-10-26" do
    changes today: "2026-11-02"

    assert_success
    assert_stdout "CHANGES SINCE 2026-10-26\n"
  end

  test "a period starts at 00:00:00 on its first day: with --since 7d on 2026-10-10, a commit made at 00:00:00 on 2026-10-03 is reported" do
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-03T00:00:00+00:00"

    changes "--since", "7d"

    assert_stdout "Hold the place while the payment is pending"
  end

  test "with --since 7d on 2026-10-10, a commit made at 23:59:59 on 2026-10-02, the day before the period starts, is not reported" do
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-02T23:59:59+00:00"

    changes "--since", "7d"

    refute_stdout "Hold the place while the payment is pending"
    assert_stdout "processes: 0 changed"
  end
end

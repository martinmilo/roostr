class Last7DaysTest < AcceptanceTest
  test "when the newest change was committed at 23:59:59 on 2026-10-02 and today is 2026-10-10, nothing changed in the last 7 days and LAST 7 DAYS is left out" do
    workspace "atelier", git: true
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-02T23:59:59+00:00"

    overview

    assert_success
    refute_stdout "LAST 7 DAYS"
  end

  test "in a Git repository with no commits yet, the overview is printed without LAST 7 DAYS and without NEEDS ATTENTION" do
    workspace "atelier"
    git "init", "--quiet"

    overview

    assert_success
    assert_stdout "7 entrypoints, 5 described"
    refute_stdout "LAST 7 DAYS"
    refute_stdout "NEEDS ATTENTION"
  end

  test "with ROOSTR_TODAY carrying a time, 2026-10-10 14:30, the last 7 days still start at 00:00 on 2026-10-03: a change committed then is in LAST 7 DAYS" do
    workspace "atelier", git: true
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-03T00:00:00+00:00"

    overview today: "2026-10-10 14:30"

    assert_success
    assert_stdout "LAST 7 DAYS"
  end
end

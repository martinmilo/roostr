class CheckNotDescribedTest < AcceptanceTest
  def setup = atelier_under_git # entrypoints.tsv holds 7; GET /bookings/:id and GET /workshops are listed by no record

  test "with --base HEAD, entrypoints a rescan added that no record lists are reported under NOT DESCRIBED with the remedy, exit 1" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv") # adds http POST /waiting_list and job WaitingListJob

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "NOT DESCRIBED\n"
    assert_stdout "\n    http POST /waiting_list\n    job WaitingListJob\n"
    assert_stdout "describe a new process for it."
  end

  test "with --base HEAD, entrypoints that no record listed already at HEAD are old debt: NOT DESCRIBED lists only the new ones" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv")

    check "--base", "HEAD"

    assert_stdout "NOT DESCRIBED\n"
    refute_stdout "GET /bookings/:id"
    refute_stdout "GET /workshops"
  end

  test "with --base HEAD, a new entrypoint that a record lists is not reported" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv")
    write_file ".roostr/processes/cancel-booking.yml", fixture("processes/cancel-booking-with-waiting-list.yml") # now also lists http POST /waiting_list and job WaitingListJob

    check "--base", "HEAD"

    assert_success
    refute_stdout "NOT DESCRIBED"
  end

  test "without --base, a new entrypoint that no record lists is not reported" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv")

    check

    assert_success
    refute_stdout "NOT DESCRIBED"
  end

  test "with --base main, an entrypoint that main removed after the branch left it is not reported as new on the branch: the base's entrypoints are read where the branch left it" do
    on_a_branch_after_main_moved_on

    check "--base", "main"

    assert_success
    refute_stdout "GET /bookings/:id"
  end

  test "with --base HEAD, when HEAD had no entrypoints.tsv, new entrypoints are not compared and none is reported" do
    delete_file ".roostr/entrypoints.tsv"
    commit "Remove entrypoints.tsv before the first scan", on: "2026-10-05" # HEAD has no entrypoints.tsv
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv") # the first scan after HEAD

    check "--base", "HEAD"

    assert_success
    refute_stdout "NOT DESCRIBED"
  end
end

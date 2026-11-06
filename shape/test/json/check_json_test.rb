class CheckJsonTest < AcceptanceTest
  test "a project with nothing to report gives base null and problems empty, and exits 0" do
    atelier_under_git # committed once, nothing uncommitted, no scan script

    check "--json"

    assert_success
    assert_json "base", nil
    assert_json "problems", []
  end

  test "a validate finding is a problem of kind invalid, with the file, key and line validate gives" do
    atelier_under_git
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/cancel-route-dropped.tsv") # http DELETE /bookings/:id, the first entrypoint of cancel-booking, is gone

    check "--json"

    assert_exit 1
    assert_json "problems.*.kind", ["invalid"]
    assert_json "problems.1.file", ".roostr/processes/cancel-booking.yml"
    assert_json "problems.1.key", "entrypoints.1"
    assert_json "problems.1.line", nil
  end

  test "a scan script and no entrypoints.tsv yet give one problem of kind entrypoints_never_written" do
    init # a new project: no entrypoints.tsv, no records
    scan_prints fixture("scans/atelier.txt")

    check "--json"

    assert_exit 1
    assert_json "problems", [{ "kind" => "entrypoints_never_written" }]
  end

  test "halfway through the waiting list, check --base HEAD --json gives base HEAD and the problems in the order of the text: entrypoints, the processes that need attention, what is not described, exactly as laid out" do
    atelier_halfway_through_the_waiting_list

    check "--base", "HEAD", "--json"

    assert_exit 1
    assert_json_is expected("check_halfway_through_the_waiting_list.json")
  end
end

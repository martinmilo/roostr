class CheckScanRunsTest < AcceptanceTest
  def setup = atelier_under_git # each test commits a fake .roostr/scan, so that HEAD has it and only the change after HEAD differs

  test "with --base HEAD, a change to a code file that is not committed makes check run the scan script and report entrypoints.tsv behind it" do
    commit_scan_prints fixture("scans/atelier-with-waiting-list.txt") # finds POST /waiting_list, which entrypoints.tsv lacks
    edit_file "app/controllers/workshops_controller.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "+ http POST /waiting_list"
  end

  test "with --base HEAD, a new code file that Git does not track yet counts as a change: the scan script runs" do
    commit_scan_prints fixture("scans/atelier-with-waiting-list.txt")
    write_file "app/controllers/waiting_list_controller.rb", "class WaitingListController < ApplicationController\nend\n"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "+ http POST /waiting_list"
  end

  test "with --base HEAD, a change that is staged and not committed counts, as a pre-commit hook sees it: the scan script runs" do
    commit_scan_prints fixture("scans/atelier-with-waiting-list.txt")
    edit_file "app/controllers/workshops_controller.rb"
    git "add", "app/controllers/workshops_controller.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "+ http POST /waiting_list"
  end

  test "with --base HEAD, a deleted code file counts as a change: the scan script runs" do
    commit_scan_prints fixture("scans/atelier-with-waiting-list.txt")
    delete_file "app/controllers/workshops_controller.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "+ http POST /waiting_list"
  end

  test "with --base HEAD, a change only to a file Git ignores, such as a log, is no change: the scan script does not run" do
    write_file ".gitignore", "log/\ntmp/\n"
    commit_scan_script "leaves-a-trace.sh" # creates scan-was-run whenever it runs
    write_file "log/development.log", "Started GET \"/workshops\" for 127.0.0.1\n"

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
    refute_file "scan-was-run"
  end

  test "with --base HEAD, a change to .roostr/scan itself makes check run the scan script" do
    commit_scan_prints fixture("scans/atelier.txt")
    scan_prints fixture("scans/atelier-with-waiting-list.txt") # the script changed and is not committed

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "+ http POST /waiting_list"
  end
end

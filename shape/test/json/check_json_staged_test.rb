class CheckJsonStagedTest < AcceptanceTest
  def setup = atelier_under_git # committed once, nothing uncommitted, no scan script

  test "check --staged --json gives base HEAD, staged true, and a file not staged as a problem of kind not_staged with its file" do
    the_waiting_list_added_with_its_record
    stage_everything_but ".roostr/processes/join-waiting-list.yml"

    check "--staged", "--json"

    assert_exit 1
    assert_json "base", "HEAD"
    assert_json "staged", true
    assert_json "problems", [{ "kind" => "not_staged", "file" => ".roostr/processes/join-waiting-list.yml" }]
  end

  test "check --base HEAD --json gives staged false" do
    check "--base", "HEAD", "--json"

    assert_success
    assert_json "staged", false
  end
end

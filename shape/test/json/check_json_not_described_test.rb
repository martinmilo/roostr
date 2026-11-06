class CheckJsonNotDescribedTest < AcceptanceTest
  test "with remind-attendees.yml deleted, check --base HEAD --json gives one problem of kind not_described for each of its entrypoints" do
    atelier_under_git
    delete_file ".roostr/processes/remind-attendees.yml"

    check "--base", "HEAD", "--json"

    assert_exit 1
    assert_json "problems", [{ "kind" => "not_described", "entrypoint" => "job BookingReminderJob" },
                             { "kind" => "not_described", "entrypoint" => "mail BookingMailer#reminder" }]
  end
end

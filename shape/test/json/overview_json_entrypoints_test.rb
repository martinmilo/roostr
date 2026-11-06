class OverviewJsonEntrypointsTest < AcceptanceTest
  def setup = workspace("atelier") # entrypoints.tsv covers http, job and mail and holds 7 rows; the two mail rows have no area

  test "entrypoints gives the kinds of the covers line, and one row per line of entrypoints.tsv, in its order" do
    overview "--json"

    assert_json "entrypoints.covers", ["http", "job", "mail"]
    assert_json "entrypoints.rows.*.entrypoint", ["http DELETE /bookings/:id", "http GET /bookings/:id", "http GET /workshops",
                                                  "http POST /bookings", "job BookingReminderJob",
                                                  "mail BookingMailer#confirmation", "mail BookingMailer#reminder"]
  end

  test "a row gives its entrypoint, kind, name, file and area, and the ids of the records that list it, in id order" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv") # http POST /waiting_list is its fifth row
    write_file ".roostr/processes/join-waiting-list.yml", fixture("processes/join-waiting-list.yml") # lists http POST /waiting_list
    write_file ".roostr/processes/cancel-booking.yml", fixture("processes/cancel-booking-with-waiting-list.yml") # lists it too

    overview "--json"

    assert_json "entrypoints.rows.5", { "entrypoint" => "http POST /waiting_list", "kind" => "http", "name" => "POST /waiting_list",
                                        "file" => "app/controllers/waiting_list_controller.rb", "area" => "Bookings",
                                        "processes" => ["cancel-booking", "join-waiting-list"] }
  end

  test "a row without an area has area null, and a row that no record lists has processes empty" do
    overview "--json"

    assert_json "entrypoints.rows.6.area", nil      # mail BookingMailer#confirmation
    assert_json "entrypoints.rows.2.processes", []  # http GET /bookings/:id
  end

  test "without entrypoints.tsv, entrypoints is null, present and not left out" do
    delete_file ".roostr/entrypoints.tsv"

    overview "--json"

    assert_success
    assert_json "entrypoints", nil
  end

  test "a row of a kind outside the closed list is not among the rows, and entrypoints.tsv is in needs_checking" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/unknown-kind.tsv") # the 7 rows of Atelier, then one of kind webhook

    overview "--json"

    assert_json "entrypoints.rows.*.kind", ["http", "http", "http", "http", "job", "mail", "mail"]
    assert_json "needs_checking", [{ "file" => ".roostr/entrypoints.tsv" }]
  end
end

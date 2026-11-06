class JsonValuesTest < AcceptanceTest
  def setup = workspace("atelier")

  test "a name with double quotes and a backslash comes back as the same string" do
    write_file ".roostr/processes/export-to-accounts.yml", fixture("processes/export-to-accounts.yml") # name: Export bookings to "Accounts\2026"

    show "export-to-accounts", "--json"

    assert_json "process.name", 'Export bookings to "Accounts\2026"'
  end

  test "a try written as a literal block, of several lines with leading spaces and a tab, comes back exactly, ending in the newline the block keeps" do
    write_file ".roostr/processes/export-to-accounts.yml", fixture("processes/export-to-accounts.yml") # the third line of try starts with a tab

    show "export-to-accounts", "--json"

    assert_json "process.try", "In a shell:\n" \
                               "  bin/rails runner 'ExportBookings.call(month: \"2026-09\")'\n" \
                               "\tThen open tmp/exports/2026-09.csv.\n"
  end

  test "a description with accented letters comes back as written" do
    write_file ".roostr/processes/export-to-accounts.yml", fixture("processes/export-to-accounts.yml")

    show "export-to-accounts", "--json"

    assert_json "process.description", "Lists the month's bookings, the crème brûlée workshop at the café included, for the accountant."
  end

  test "a why item with an emoji comes back as written" do
    write_file ".roostr/processes/export-to-accounts.yml", fixture("processes/export-to-accounts.yml")

    show "export-to-accounts", "--json"

    assert_json "process.why", ["Accounts asked for one file a month 📅 so that nothing is entered twice."]
  end

  test "a checked date comes back as the string 2026-10-08" do
    write_file ".roostr/processes/remind-attendees.yml", fixture("processes/remind-attendees-checked.yml") # the same record with checked: 2026-10-08

    show "remind-attendees", "--json"

    assert_json "process.checked", "2026-10-08"
  end

  test "a record with only a name and a description still has every key: group, try and checked are null, and the lists are empty" do
    write_file ".roostr/processes/export-bookings.yml", "name: Export bookings\ndescription: Export the month's bookings for the accountant.\n"

    show "export-bookings", "--json"

    assert_json "process", { "id" => "export-bookings", "name" => "Export bookings",
                             "description" => "Export the month's bookings for the accountant.", "group" => nil,
                             "entrypoints" => [], "files" => [], "try" => nil, "outcomes" => [], "why" => [], "checked" => nil }
  end
end

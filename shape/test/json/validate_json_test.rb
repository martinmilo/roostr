class ValidateJsonTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes, 7 entrypoints, every listed file present

  test "a project with no problems gives problems empty and exits 0" do
    validate "--json"

    assert_success
    assert_json "problems", []
  end

  test "a record that lists an entrypoint not in entrypoints.tsv gives one problem of kind invalid with its file, key entrypoints.1, line null and a message naming the entrypoint, and exits 1 with the JSON on stdout" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists http POST /refunds

    validate "--json"

    assert_exit 1
    assert_json "problems.*.kind", ["invalid"]
    assert_json "problems.1.file", ".roostr/processes/refund-booking.yml"
    assert_json "problems.1.key", "entrypoints.1"
    assert_json "problems.1.line", nil
    assert_json "problems.1.message", /http POST \/refunds/
  end

  test "a record that does not parse gives a problem with its file, and key and line null" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"

    validate "--json"

    assert_json "problems.1.file", ".roostr/processes/cancel-booking.yml"
    assert_json "problems.1.key", nil
    assert_json "problems.1.line", nil
  end

  test "an entrypoints.tsv row of a kind outside the closed list gives a problem with file .roostr/entrypoints.tsv, key null and the row's line, 9" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/unknown-kind.tsv") # the header, the 7 rows of Atelier, then a row of kind webhook on line 9

    validate "--json"

    assert_json "problems.1.file", ".roostr/entrypoints.tsv"
    assert_json "problems.1.key", nil
    assert_json "problems.1.line", 9
  end

  test "a missing entrypoints.tsv, while records list entrypoints, gives one problem with file .roostr/entrypoints.tsv, and key and line null" do
    delete_file ".roostr/entrypoints.tsv"

    validate "--json"

    assert_json "problems.*.file", [".roostr/entrypoints.tsv"]
    assert_json "problems.1.key", nil
    assert_json "problems.1.line", nil
  end

  test "refund-booking.yaml, a file roostr does not read as a record, gives a problem of kind invalid with its file, and key and line null" do
    write_file ".roostr/processes/refund-booking.yaml", fixture("processes/refund-booking.yml")

    validate "--json"

    assert_exit 1
    assert_json "problems.*.kind", ["invalid"]
    assert_json "problems.1.file", ".roostr/processes/refund-booking.yaml"
    assert_json "problems.1.key", nil
    assert_json "problems.1.line", nil
  end
end

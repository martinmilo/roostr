class JsonObjectTest < AcceptanceTest
  def setup = atelier_under_git # committed once, nothing uncommitted, no scan script: nothing to report

  test "roostr --json prints the overview as exactly one JSON object on stdout and nothing else, with format 1" do
    overview "--json"

    assert_success
    assert_json "format", 1
  end

  test "show book-workshop --json prints exactly one JSON object on stdout and nothing else, with format 1" do
    show "book-workshop", "--json"

    assert_success
    assert_json "format", 1
  end

  test "changes --json prints exactly one JSON object on stdout and nothing else, with format 1" do
    changes "--json"

    assert_success
    assert_json "format", 1
  end

  test "validate --json prints exactly one JSON object on stdout and nothing else, with format 1" do
    validate "--json"

    assert_success
    assert_json "format", 1
  end

  test "check --json prints exactly one JSON object on stdout and nothing else, with format 1" do
    check "--json"

    assert_success
    assert_json "format", 1
  end
end

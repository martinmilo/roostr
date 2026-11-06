class ShowJsonTest < AcceptanceTest
  def setup = workspace("atelier") # book-workshop: two entrypoints, two files with one note, a three-line try, two outcomes, two why paragraphs

  test "show book-workshop --json gives the process with every key, and each of its entrypoints with its file, exactly as laid out, and writes nothing" do
    show "book-workshop", "--json"

    assert_success
    assert_json_is expected("show_book_workshop.json")
    assert_nothing_written
  end

  test "in show --json, an entrypoint that is not in entrypoints.tsv has file null" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking.yml") # lists http POST /refunds, which the scan did not find

    show "refund-booking", "--json"

    assert_success
    assert_json "entrypoints", [{ "entrypoint" => "http POST /refunds", "file" => nil }]
  end
end
